use crate::{
    filter::{self, GlobalFilter},
    filter_data_panel::FilterDataPanel,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct FilterPanel {
    pub global_filter: Rc<RefCell<GlobalFilter>>,
    pub data_panel: FilterDataPanel,
    pub stop: bool,
}

impl FilterPanel {
    pub fn new(global_filter: Rc<RefCell<GlobalFilter>>) -> Self {
        let data_panel = FilterDataPanel::new(global_filter.borrow().data.clone());
        Self {
            data_panel,
            global_filter,
            stop: false,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> Option<FilterDataPanel> {
        let mut changed = false;
        ui.horizontal(|ui| {
            if ui
                .button(if self.stop { "START" } else { "STOP " })
                .clicked()
            {
                self.stop = !self.stop;
                changed = true;
            }

            if ui.button("ALL").clicked() {
                changed = true;
                self.global_filter.borrow_mut().ignore_type = filter::Flags::NONE;
            }

            if ui.button("NONE").clicked() {
                changed = true;
                self.global_filter.borrow_mut().ignore_type = filter::Flags::ALL;
            }

            let f = self.global_filter.borrow_mut().ignore_type;
            for (name, val) in filter::Flags::all().iter_names() {
                let mut flag = (f & val) == val;
                flag = !flag; // invert checkbox value
                if ui.checkbox(&mut flag, name).changed() {
                    changed = true;
                    if flag {
                        self.global_filter.borrow_mut().ignore_type.remove(val);
                    } else {
                        self.global_filter.borrow_mut().ignore_type.insert(val);
                    }
                }
            }
        });

        let mut to_add_fixed_filter: Option<FilterDataPanel> = None;
        ui.horizontal(|ui| {
            changed |= self.data_panel.update(ui);
            if ui
                .button("➕")
                .on_hover_text(
                    "Pin new filter which will show only last filtered message in table below",
                )
                .clicked()
            {
                to_add_fixed_filter = Some(self.data_panel.clone());
            }
        });

        to_add_fixed_filter
    }
}
