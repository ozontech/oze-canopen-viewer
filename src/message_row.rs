use crate::message_cached::MessageCached;
use oze_canopen::canopen::RxMessageToStringFormat;
use tokio::time::Instant;

#[derive(Debug)]
pub struct MessageRow {
    pub start_time: Instant,
    pub format: RxMessageToStringFormat,
}

impl Default for MessageRow {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            format: RxMessageToStringFormat::Hex,
        }
    }
}

impl MessageRow {
    pub fn header(&self, ui: &mut egui::Ui) {
        self.header_custom(ui, "     Timestamp");
    }

    pub fn header_custom(&self, ui: &mut egui::Ui, time: &str) {
        ui.label(time);
        ui.label("COB ID");
        ui.label(match self.format {
            RxMessageToStringFormat::Binary => {
                " Binary data                                                            "
                //00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            }
            RxMessageToStringFormat::Hex => {
                " Hex data               "
                //00 00 00 00 00 00 00 00
            }
            RxMessageToStringFormat::Ascii => "ASCII data",
            RxMessageToStringFormat::Utf8 => "UTF8 data",
        });

        ui.label("Packet type");
        ui.label("Node ID");
        ui.label("Info");
    }

    pub fn message(&self, ui: &mut egui::Ui, d: &MessageCached) {
        self.message_custom_timestamp(ui, d, &self.start_time);
    }

    pub fn message_custom_timestamp(&self, ui: &mut egui::Ui, d: &MessageCached, time: &Instant) {
        let desc = d.msg.parsed_type.to_string();

        let time = d.get_timestamp().duration_since(*time).as_secs_f32();
        let time = format!("{time:.6}");
        let cob = &d.cob_str;
        let data = d.get_by_format(self.format);
        let node_id = if let Some(node_id) = d.msg.parsed_node_id {
            format!("{node_id:3}")
        } else {
            "   ".to_owned()
        };

        ui.label(time);
        ui.label(cob);
        ui.label(data).on_hover_ui(|ui| {
            // data in all formats on hover
            ui.label(format!("HEX:   {}", d.hex_str));
            ui.label(format!("BIN:   {}", d.bin_str));
            ui.label(format!("ASCII: {}", d.ascii_str));
        });
        ui.label(desc);
        ui.label(node_id);
        ui.label(d.additional.to_string())
            .on_hover_text_at_pointer(d.additional.get_tooltip());
    }
}
