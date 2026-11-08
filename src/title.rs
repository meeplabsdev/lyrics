use eframe::egui::{Align, Color32, Label, RichText, Widget};

pub struct Title {}

impl Title {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Title {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(
            Label::new(RichText::new("Among Us Impostor").color(Color32::WHITE))
                .halign(Align::Center)
                .selectable(false),
        )
    }
}
