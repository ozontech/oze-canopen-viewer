use std::sync::Arc;

use crate::{bitrate::RatesData, theme::OZON_PINK};
use egui::Vec2b;
use egui_plot::{Line, Plot, PlotPoints};
use tokio::{runtime::Handle, sync::Mutex};

#[derive(Debug)]
pub struct Chart {
    channel: Arc<Mutex<RatesData>>,
}

impl Chart {
    pub fn new(channel: Arc<Mutex<RatesData>>) -> Chart {
        Chart { channel }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let plot = Plot::new("plot")
            .height(150.0)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .show_axes(Vec2b::new(false, true));

        Handle::current().block_on(async {
            let data: Vec<[f64; 2]> = self.channel.lock().await.clone();
            // There is no Borrowed PlotPoints so we need to copy every time
            plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::new(data)).color(OZON_PINK));
            })
        });
    }
}
