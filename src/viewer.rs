use crate::{filter::GlobalFilter, message_cached::MessageCached, message_row::MessageRow};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

#[derive(Debug)]
pub struct Viewer {
    global_filter: Rc<RefCell<GlobalFilter>>,
    pub message_row: MessageRow,
}

impl Viewer {
    pub fn new(global_filter: Rc<RefCell<GlobalFilter>>) -> Self {
        Self {
            message_row: MessageRow::default(),
            global_filter,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, data: &VecDeque<MessageCached>) {
        let filt = self.global_filter.borrow();
        let data: VecDeque<&MessageCached> = data.iter().filter(|i| !filt.filter(i)).collect();

        // let sessions: Vec<(u8, u8)> = Vec::new();
        // for i in &data {
        //     match &i.additional {
        //         RxMessageAdditional::SdoTx(d) => (),
        //         RxMessageAdditional::SdoRx(d) => (),

        //         _ => (),
        //     }
        // }

        let row_spacing = 4.0;
        let column_spacing = 20.0;
        let text_style = egui::TextStyle::Body;
        let text_height = ui.text_style_height(&text_style);
        let height = text_height + row_spacing;
        egui::ScrollArea::vertical().animated(true).show_rows(
            ui,
            height,
            data.len() + 1,
            |ui, row_range| {
                egui::Grid::new("viewer_grid")
                    .start_row(row_range.start)
                    .spacing([column_spacing, row_spacing])
                    .striped(true)
                    .min_row_height(height)
                    .show(ui, |ui| {
                        let data_range = if row_range.start == 0 {
                            self.message_row.header(ui);
                            ui.end_row();
                            0..(row_range.end - 1)
                        } else {
                            (row_range.start - 1)..(row_range.end - 1)
                        };

                        for d in data.range(data_range) {
                            self.message_row.message(ui, d);
                            ui.end_row();
                        }

                        // Костыль нужный, чтобы выровнять ширину столбца
                        self.message_row.header(ui);
                        ui.end_row();
                    });
            },
        );
    }
}
