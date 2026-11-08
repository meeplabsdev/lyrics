use eframe::egui::{Color32, Widget};

pub struct Lyrics {}

impl Lyrics {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Lyrics {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.painter()
            .rect_filled(ui.available_rect_before_wrap(), 0, Color32::BLACK);
        ui.response()
    }
}
