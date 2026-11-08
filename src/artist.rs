use eframe::egui::{Align, Color32, FontFamily, Label, RichText, Widget};

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
        ui.add(
            Label::new(
                RichText::new(format!("{:04} - {}", self.year, self.artist))
                    .color(Color32::WHITE)
                    .family(FontFamily::Name("lyricsregular".into()))
                    .size(ui.available_height() * 0.75),
            )
            .halign(Align::Center)
            .selectable(false),
        )
    }
}
