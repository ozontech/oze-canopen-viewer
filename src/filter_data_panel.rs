use crate::filter::DataFilter;
use egui::TextEdit;
use regex::Regex;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct FilterDataPanel {
    pub data_filter: Rc<RefCell<DataFilter>>,
    node_raw: String,
    regex_raw: String,
    regex_cob_raw: String,
}

impl FilterDataPanel {
    pub fn new(data_filter: Rc<RefCell<DataFilter>>) -> Self {
        Self {
            data_filter,
            regex_raw: String::new(),
            regex_cob_raw: String::new(),
            node_raw: String::new(),
        }
    }

    /// # Panics
    pub fn update(&mut self, ui: &mut egui::Ui) -> bool {
        let mut data_filter = self.data_filter.try_borrow_mut().unwrap();
        let mut changed = false;
        if ui
            .add(
                TextEdit::singleline(&mut self.regex_cob_raw)
                    .hint_text("cob regex")
                    .desired_width(100.0),
            )
            .on_hover_text("You can use complex regex filters here. For exemple: '^.. 0A ..'")
            .changed()
        {
            changed = true;
            data_filter.regex_cob = Regex::new(&self.regex_cob_raw).ok();
        }
        if ui
            .add(
                TextEdit::singleline(&mut self.node_raw)
                    .hint_text("nodeID")
                    .desired_width(55.0),
            )
            .on_hover_text("This fields needs to be an decimal intenger representing NodeID")
            .changed()
        {
            changed = true;
            data_filter.node_id = self.node_raw.parse().ok();
        }
        if ui
            .add(
                TextEdit::singleline(&mut self.regex_raw)
                    .hint_text("hex data regex")
                    .desired_width(200.0),
            )
            .on_hover_text("You can use complex regex filters here. For exemple: '^.. 0A ..'")
            .changed()
        {
            changed = true;
            data_filter.regex = Regex::new(&self.regex_raw).ok();
        }
        ui.separator();
        changed
    }
}
