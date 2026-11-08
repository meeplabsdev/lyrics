use eframe::egui::{Image, Widget, include_image};

pub struct Artwork {}

impl Artwork {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Artwork {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(Image::new(include_image!("../assets/test.png")).corner_radius(4))
    }
}
