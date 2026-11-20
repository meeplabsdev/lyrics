use crate::colour::Colour;
use eframe::egui::{Align2, FontFamily, FontId, Sense, Widget};

pub struct QrInstructions {
    code: String,
}

impl QrInstructions {
    pub fn new() -> Self {
        Self {
            code: String::new(),
        }
    }

    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }
}

impl Widget for &mut QrInstructions {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let font_id_big = FontId::new(
            ui.available_height() * 0.5,
            FontFamily::Name("lyricsbold".into()),
        );

        let font_id = FontId::new(
            ui.available_height() * 0.3,
            FontFamily::Name("lyricsregular".into()),
        );

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::empty());

        if self.code.is_empty() {
            return response;
        }

        ui.painter().text(
            rect.center_top(),
            Align2::CENTER_TOP,
            "Scan code to log in",
            font_id_big,
            Colour::WHITE.into(),
        );

        ui.painter().text(
            rect.center_bottom(),
            Align2::CENTER_BOTTOM,
            format!("Go to spotify.com/pair and enter {}", self.code.clone()),
            font_id,
            Colour::WHITE.into(),
        );

        response
    }
}
