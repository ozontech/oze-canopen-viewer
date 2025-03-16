use crate::{
    filter_data_panel::FilterDataPanel, message_cached::MessageCached, message_row::MessageRow,
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use tokio::time::Instant;

#[derive(Debug, Default)]
pub struct PinnedFilters {
    data: Vec<(FilterDataPanel, Instant, Option<MessageCached>)>,
    pub message_row: MessageRow,
}

impl PinnedFilters {
    pub fn pin_filter(&mut self, mut filt: FilterDataPanel, data: &VecDeque<MessageCached>) {
        let data_filter = filt.data_filter.borrow().clone();
        let new_data = data.iter().find(|i| !data_filter.filter(i));

        filt.data_filter = Rc::new(RefCell::new(data_filter));
        self.data.push((filt, Instant::now(), new_data.cloned()));
    }

    pub fn push_data(&mut self, msg: &MessageCached) {
        for data in &mut self.data {
            if !data.0.data_filter.borrow().filter(msg) {
                let time = data.2.clone().map_or(Instant::now(), |x| x.get_timestamp());
                data.1 = time;
                data.2 = Some(msg.clone());
            }
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        let row_spacing = 4.0;
        let column_spacing = 5.0;
        let text_style = egui::TextStyle::Body;
        let text_height = ui.text_style_height(&text_style);
        let height = text_height + row_spacing;
        egui::Grid::new("fixed_grid")
            .spacing([column_spacing, row_spacing])
            .striped(true)
            .min_row_height(height)
            .show(ui, |ui| {
                ui.label("🗑");
                ui.label("Filter");
                self.message_row.header_custom(ui, "   Time delta ");
                ui.end_row();

                let mut to_delete: Option<usize> = None;
                for (index, (filt, time, msg)) in &mut self.data.iter_mut().enumerate() {
                    if ui.button("❌").clicked() {
                        to_delete = Some(index);
                    }
                    ui.horizontal(|ui| filt.update(ui));
                    if let Some(msg) = msg {
                        self.message_row.message_custom_timestamp(ui, msg, time);
                    }
                    ui.end_row();
                }

                if let Some(index) = to_delete {
                    self.data.remove(index);
                }
            });
    }
}
