use clap::Parser;
use oze_canopen::interface::Connection;
use oze_canopen_viewer::bitrate;
use oze_canopen_viewer::driver::{self, Control};
use oze_canopen_viewer::gui::Gui;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::{watch, Mutex};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    can: Option<String>,
    #[arg(short, long)]
    bitrate: Option<u32>,
}

fn main() -> eframe::Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    let initial_control = Control {
        command: driver::ControlCommand::Process,
        connection: Connection {
            can_name: args.can.clone().unwrap_or_default(),
            bitrate: args.bitrate,
        },
    };

    let (state_snd, state_rcv) = watch::channel(driver::State::default());
    let (ctrl_snd, ctrl_rcv) = watch::channel(initial_control.clone());

    let bitrates = Arc::new(Mutex::new(Vec::new()));
    let bitrates_thr = bitrates.clone();
    let ctrl_snd_thr = ctrl_snd.clone();
    let rt = Runtime::new().expect("Unable to create Runtime");

    let _enter = rt.enter();

    thread::spawn(move || {
        rt.block_on(async {
            let drv = driver::Driver::new(state_snd, ctrl_rcv);
            let br = bitrate::Bitrate::new(drv.co.info.clone(), bitrates_thr.clone());
            drv.start_thread();
            br.start_thread();

            if let Some(can_name) = args.can {
                println!("Use args: {can_name:?} {:?}", args.bitrate);
                ctrl_snd_thr.send(initial_control).unwrap();
            }

            tokio::signal::ctrl_c().await
        })
        .unwrap();
    });

    let native_options = eframe::NativeOptions {
        viewport: {
            egui::ViewportBuilder::default()
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0])
        },
        ..Default::default()
    };

    eframe::run_native(
        "OZON CanOpen Viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(Gui::new(cc, state_rcv, ctrl_snd, bitrates)))),
    )
}
