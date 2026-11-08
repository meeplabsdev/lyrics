use eframe::egui::{Align, Color32, FontFamily, Label, RichText, Widget};

pub struct Title {
    title: String,
}

impl Title {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}

impl Widget for &mut Title {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(
            Label::new(
                RichText::new(&self.title)
                    .color(Color32::WHITE)
                    .family(FontFamily::Name("lyricsbold".into()))
                    .size(ui.available_height() * 0.75),
            )
            .halign(Align::Center)
            .selectable(false),
        )
    }
}
