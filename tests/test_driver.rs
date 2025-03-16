#[cfg(test)]
mod tests {
    use std::time::Duration;

    use oze_canopen::{
        canopen,
        interface::Connection,
        proto::nmt::{NmtCommand, NmtCommandSpecifier},
    };
    use oze_canopen_viewer::driver::{self, Control};
    use tokio::{sync::watch, time::sleep};

    async fn send_test_messages() {
        let (interface, mut handles) = canopen::start(String::from("vcan0"), None);
        sleep(Duration::from_secs(1)).await;

        interface
            .send_nmt(NmtCommand::new(NmtCommandSpecifier::StartRemoteNode, 0))
            .await
            .unwrap();

        sleep(Duration::from_millis(500)).await;
        handles.close_and_join().await;
    }

    #[tokio::test]
    async fn test_driver_start() {
        let initial_control = Control {
            command: driver::ControlCommand::Process,
            connection: Connection {
                can_name: "vcan0".to_owned(),
                bitrate: Some(100_000),
            },
        };

        let (state_snd, state_rcv) = watch::channel(driver::State::default());
        let (ctrl_snd, ctrl_rcv) = watch::channel(initial_control.clone());
        let drv = driver::Driver::new(state_snd, ctrl_rcv);
        let driver_handle = drv.start_thread();

        sleep(Duration::from_millis(100)).await;
        assert!(!state_rcv.borrow().exit_signal);

        send_test_messages().await;

        assert!(!state_rcv.borrow().data.is_empty());
        assert_eq!(state_rcv.borrow().data.back().unwrap().hex_str, "01 00");

        ctrl_snd
            .send(Control {
                command: driver::ControlCommand::Kill,
                ..initial_control
            })
            .unwrap();

        driver_handle.await.unwrap();
        assert!(state_rcv.borrow().exit_signal);
    }
}
