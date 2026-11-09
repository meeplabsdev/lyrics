use core::f32;
use std::time::Duration;

use eframe::egui::{Color32, FontFamily, FontId, Pos2, Sense, Widget};

pub struct Artist {
    year: u16,
    artist: String,
}

impl Artist {
    pub fn new(year: u16, artist: String) -> Self {
        Self { year, artist }
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
            format!("{:04} - {}", self.year, self.artist),
            font_id,
            Color32::WHITE,
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

        ui.painter().galley(text_pos, galley, Color32::WHITE);

        if overflow > 0.0 {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }

        response
    }
}
