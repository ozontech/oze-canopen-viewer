use crate::message_cached::MessageCached;
use oze_canopen::{
    canopen::{self, JoinHandles},
    interface::{CanOpenInfo, CanOpenInterface, Connection},
};
use std::{collections::VecDeque, time::Duration};
use tokio::{signal::ctrl_c, sync::watch, task::JoinHandle, time::sleep};

/// Enum representing different control commands that can be sent to the driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlCommand {
    Stop,
    Kill,
    Process,
}

/// Struct representing the state of the CAN interface and received messages.
#[derive(Default, Debug, Clone)]
pub struct State {
    pub can_name: String,
    pub bitrate: Option<u32>,
    pub data: VecDeque<MessageCached>,
    pub info: CanOpenInfo,
    pub exit_signal: bool,
}

/// Struct representing control data including the command and connection details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    pub command: ControlCommand,
    pub connection: Connection,
}

/// Struct representing the driver responsible for processing CAN messages and handling control commands.
pub struct Driver {
    sender: watch::Sender<State>,
    receiver: watch::Receiver<Control>,
    state: State,
    pub co: CanOpenInterface,
    control: Control,
    index: u64,
    handles: JoinHandles,
}

const MAX_MESSAGES_IN_STATE: usize = 512;

impl Driver {
    pub fn new(sender: watch::Sender<State>, receiver: watch::Receiver<Control>) -> Self {
        // Initialize the CANopen interface with the initial connection details.
        let initial_connection = receiver.borrow().connection.clone();
        let (co, handles) = canopen::start(initial_connection.can_name, initial_connection.bitrate);

        // Create the driver and start running it.
        let control = receiver.borrow().clone();
        Driver {
            co,
            sender,
            control,
            receiver,
            index: 0,
            state: State::default(),
            handles,
        }
    }

    /// Asynchronously processes incoming CAN messages and control commands.
    async fn process(&mut self) {
        // Wait for a message, timeout, or ctrl_c signal.
        let rcv = tokio::select! {
            rcv = self.co.rx.recv() => Some(rcv),
            () = sleep(Duration::from_millis(100)) => None,
            _ = ctrl_c() => {
                self.control.command = ControlCommand::Kill;
                return;
            },
        };

        // Get the latest control data if it has changed.
        if self.receiver.has_changed().unwrap() {
            self.control = self.receiver.borrow_and_update().clone();
            // Update connection details if they have changed.
            self.co
                .connection
                .lock()
                .await
                .clone_from(&self.control.connection);
        }

        // Set information from the CANopen stack to the state.
        let info = self.co.info.lock().await.clone();
        self.state.info = info;

        // Handle control commands.
        match self.control.command {
            ControlCommand::Stop | ControlCommand::Kill => {
                return;
            }
            ControlCommand::Process => {}
        }

        // If no message has been received, return.
        let Some(Ok(d)) = rcv else {
            return;
        };

        // Parse and cache the received message.
        let d = MessageCached::new(self.index, d);
        self.index += 1;

        // Add the new message to the state, ensuring the state does not exceed the max size.
        while self.state.data.len() > MAX_MESSAGES_IN_STATE {
            self.state.data.pop_front();
        }
        self.state.data.push_back(d);
    }

    /// Asynchronously runs the driver, continuously processing messages and sending state updates.
    async fn run(&mut self) {
        self.state.data.reserve_exact(MAX_MESSAGES_IN_STATE);
        loop {
            self.process().await;
            if self.control.command == ControlCommand::Kill {
                self.state.exit_signal = true;
            }

            self.sender.send(self.state.clone()).unwrap();
            // Exit the loop if a Kill command is received.
            if self.control.command == ControlCommand::Kill {
                break;
            }
        }
    }

    /// Starts the driver with the given state and control channels.
    pub fn start_thread(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
            self.handles.close_and_join().await;
        })
    }
}
