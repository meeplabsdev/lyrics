use crate::colour::Colour;
use core::f32;
use eframe::egui::{FontFamily, FontId, Pos2, Sense, Widget};
use std::time::Duration;

pub struct Artist {
    name: String,
    year: u32,
}

impl Artist {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            year: 0000,
        }
    }

    pub fn set_artist(&mut self, name: String, year: u32) {
        self.name = name;
        self.year = year;
    }
}

impl Widget for &mut Artist {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let font_id = FontId::new(
            ui.available_height() * 0.75,
            FontFamily::Name("lyricsregular".into()),
        );

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::hover());

        let galley = ui.painter().layout_no_wrap(
            format!("{:04} - {}", self.year, self.name),
            font_id,
            Colour::WHITE.into(),
        );

        let text_size = galley.size();
        let overflow = text_size.x - rect.width();

        let text_pos = if overflow <= 0.0 {
            Pos2::new(
                rect.center().x - text_size.x * 0.5,
                rect.center().y - text_size.y * 0.5,
            )
        } else {
            let speed = 30.0;
            let period = overflow / speed;

            let t = ui.ctx().input(|i| i.time) as f32;
            let progress = 0.5 - 0.5 * (t * f32::consts::TAU / (period * 2.0)).cos();
            let offset_x = -overflow * progress;

            Pos2::new(rect.left() + offset_x, rect.center().y - text_size.y * 0.5)
        };

        ui.painter().galley(text_pos, galley, Colour::WHITE.into());

        if overflow > 0.0 {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }

        response
    }
}
