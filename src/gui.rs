use crate::{
    bitrate::RatesData,
    chart::{self, Chart},
    driver::{Control, ControlCommand, State},
    filter::GlobalFilter,
    filter_panel::FilterPanel,
    message_cached::MessageCached,
    pinned_filter::PinnedFilters,
    theme::{theme, OZON_GRAY, OZON_PINK},
    viewer::Viewer,
};
use egui::{emath::Numeric, Button, Layout, TextEdit, Ui};
use oze_canopen::{
    canopen::RxMessageToStringFormat,
    interface::{CanOpenInfo, Connection},
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};
use tokio::{
    sync::{watch, Mutex},
    time::Instant,
};

const MESSAGES_COUNT: usize = 4096;

pub struct Gui {
    data: VecDeque<MessageCached>,
    driver: watch::Receiver<State>,
    pinned_filters: PinnedFilters,
    viewer: Viewer,
    chart: chart::Chart,
    last: Instant,
    fps: VecDeque<f64>,
    global_filter: Rc<RefCell<GlobalFilter>>,
    filter_panel: FilterPanel,

    format: RxMessageToStringFormat,

    can_name_raw: String,
    bitrate_raw: String,

    info: CanOpenInfo,

    connection: Connection,
    stopped: bool,
    driver_ctrl: watch::Sender<Control>,
}

impl Gui {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        driver: watch::Receiver<State>,
        driver_ctrl: watch::Sender<Control>,
        bitrate: Arc<Mutex<RatesData>>,
    ) -> Self {
        theme(&cc.egui_ctx);

        let global_filter = Rc::new(RefCell::new(GlobalFilter::default()));
        let connection_data = driver_ctrl.subscribe().borrow().connection.clone();
        let can_name_raw = connection_data.can_name.clone();
        let bitrate_raw = connection_data
            .bitrate
            .map(|b| b.to_string())
            .unwrap_or_default();

        Self {
            fps: VecDeque::new(),
            data: VecDeque::new(),
            pinned_filters: PinnedFilters::default(),
            info: CanOpenInfo::default(),
            connection: connection_data,
            format: RxMessageToStringFormat::Hex,
            viewer: Viewer::new(global_filter.clone()),
            filter_panel: FilterPanel::new(global_filter.clone()),
            last: Instant::now(),
            chart: Chart::new(bitrate),
            stopped: false,
            global_filter,
            can_name_raw,
            bitrate_raw,
            driver_ctrl,
            driver,
        }
    }

    fn send_driver_control(&self) {
        let _ = self.driver_ctrl.send(Control {
            command: if self.stopped {
                ControlCommand::Stop
            } else {
                ControlCommand::Process
            },
            connection: self.connection.clone(),
        });
    }

    fn get_data_from_driver(&mut self) -> bool {
        let driver = self.driver.borrow();
        for i in &driver.data {
            if let Some(last) = self.data.front() {
                if i.index <= last.index {
                    continue;
                }
            }

            self.pinned_filters.push_data(i);
            if !self.global_filter.borrow().filter(i) {
                self.data.push_front(i.clone());
            }
        }

        while self.data.len() > MESSAGES_COUNT {
            self.data.pop_back();
        }

        self.info = driver.info.clone();

        driver.exit_signal
    }

    fn calc_fps(&mut self) -> f64 {
        let fps = 1.0 / self.last.elapsed().as_secs_f64();
        self.last = Instant::now();

        self.fps.push_back(fps);

        let fps = self.fps.iter().sum::<f64>() / self.fps.len().to_f64();
        while self.fps.len() > usize::from_f64(fps.round()) * 5 {
            self.fps.pop_front();
        }

        fps.round()
    }

    fn show_connect_ui(&mut self, ui: &mut Ui) {
        ui.add(
            TextEdit::singleline(&mut self.can_name_raw)
                .hint_text("can name")
                .desired_width(100.0),
        );

        ui.add(
            TextEdit::singleline(&mut self.bitrate_raw)
                .hint_text("bitrate")
                .desired_width(100.0),
        );
        let bitrate = self.bitrate_raw.parse::<u32>().ok();
        let button_enbled = !self.can_name_raw.is_empty()
            && ((bitrate.is_some()
                && bitrate.unwrap_or_default() <= 1_000_000
                && bitrate.unwrap_or_default() > 0)
                || self.bitrate_raw.is_empty());
        if ui
            .add_enabled(button_enbled, Button::new("🔌Connect"))
            .clicked()
        {
            self.connection.can_name = self.can_name_raw.clone();
            self.connection.bitrate = bitrate;
            self.send_driver_control();
        }
    }

    fn show_format_ui(&mut self, ui: &mut Ui) {
        if ui
            .selectable_label(self.format == RxMessageToStringFormat::Hex, "hex")
            .on_hover_text("Use HEX format to show message data")
            .clicked()
        {
            self.format = RxMessageToStringFormat::Hex;
        }
        if ui
            .selectable_label(self.format == RxMessageToStringFormat::Binary, "bin")
            .on_hover_text("Use binary format to show message data")
            .clicked()
        {
            self.format = RxMessageToStringFormat::Binary;
        }
        if ui
            .selectable_label(self.format == RxMessageToStringFormat::Ascii, "ascii")
            .on_hover_text("Use ASCII encoding to show message data")
            .clicked()
        {
            self.format = RxMessageToStringFormat::Ascii;
        }
    }

    fn show_connection_help(ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
                    ui.colored_label(OZON_PINK, "↑ You need to enter can name, i.e.");
                    ui.colored_label(OZON_GRAY, "can0");
                    ui.colored_label(OZON_PINK, "and optionally bitrate. If bitrate is set then link will go down, bitrate will be changed and then link will be set up.");
                });
        ui.colored_label(OZON_PINK, "Or your CAN interface is not connected properly");
        ui.label("Or you can execute program with arguments default values, for help execute:");
        ui.colored_label(OZON_GRAY, "oze-canopen-viewer --help");
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let fps = self.calc_fps();
        let connected =
            self.info.receiver_socket || self.info.transmitter_socket || self.info.rx_bits > 0;
        if self.get_data_from_driver() {
            println!("Gracefull shutdown");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx.request_repaint();
            return;
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.show_connect_ui(ui);
                ui.separator();

                self.show_format_ui(ui);
                ui.separator();

                ui.label(format!(
                    "rx {} tx {}",
                    self.info.receiver_socket, self.info.transmitter_socket,
                ));

                ui.separator();
                ui.label(format!("packets={}", self.data.len()));

                ui.with_layout(Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(format!("{fps} FPS",));
                });
            });

            if !connected {
                Self::show_connection_help(ui);
            }
        });

        self.viewer.message_row.format = self.format;
        self.pinned_filters.message_row.format = self.format;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(connected, |ui| {
                self.chart.ui(ui);
                ui.separator();
                let to_pin = self.filter_panel.update(ui);
                if self.stopped != self.filter_panel.stop {
                    self.stopped = self.filter_panel.stop;
                    self.send_driver_control();
                }
                if let Some(to_pin) = to_pin {
                    self.pinned_filters.pin_filter(to_pin, &self.data);
                }

                ui.separator();
                self.pinned_filters.update(ui);
                ui.separator();
                self.viewer.update(ui, &self.data);
            });
        });

        ctx.request_repaint();
    }
}
