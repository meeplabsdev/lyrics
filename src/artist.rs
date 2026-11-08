use eframe::egui::{Align, Color32, Label, RichText, Widget};

pub struct Artist {}

impl Artist {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Artist {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(
            Label::new(RichText::new("Real Crewmate").color(Color32::WHITE))
                .halign(Align::Center)
                .selectable(false),
        )
    }
}
