use catppuccin_egui::{Theme, FRAPPE};
use egui::{style::Selection, Color32, FontFamily, FontId, TextStyle};
use std::ops::Deref;
pub const OZON_BLUE: Color32 = egui::Color32::from_rgb(0, 91, 255);
pub const OZON_BLUE_ACTIVE: Color32 = egui::Color32::from_rgb(30, 144, 255);
pub const OZON_PINK: Color32 = Color32::from_rgb(249, 17, 85);
pub const OZON_GRAY: Color32 = egui::Color32::from_rgb(245, 247, 255);
pub const OZON_THEME: Theme = Theme {
    blue: OZON_BLUE,
    red: OZON_PINK,
    //    surface0: OZON_GRAY, // open, inactive
    surface1: OZON_BLUE,        // active
    surface2: OZON_BLUE_ACTIVE, // hover
    overlay1: OZON_BLUE,
    ..FRAPPE
};

pub fn theme(ctx: &egui::Context) {
    setup_custom_fonts(ctx);
    configure_text_styles(ctx);
    catppuccin_egui::set_theme(ctx, OZON_THEME);

    let mut style = ctx.style().deref().clone();
    style.visuals.selection = Selection {
        bg_fill: OZON_BLUE,
        stroke: egui::Stroke {
            color: OZON_BLUE,
            width: 0.0,
        },
    };
    ctx.set_style(style);
}

fn configure_text_styles(ctx: &egui::Context) {
    use FontFamily::Monospace;
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Monospace)),
        (TextStyle::Body, FontId::new(16.0, Monospace)),
        (TextStyle::Monospace, FontId::new(16.0, Monospace)),
        (TextStyle::Button, FontId::new(16.0, Monospace)),
        (TextStyle::Small, FontId::new(8.0, Monospace)),
    ]
    .into();
    ctx.set_style(style);
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "Hack".to_owned(),
        egui::FontData::from_static(include_bytes!("./fonts/Hack-Regular.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "Hack".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("Hack".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
