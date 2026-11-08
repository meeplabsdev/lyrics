use eframe::egui::Widget;

pub struct Lyrics {}

impl Lyrics {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Lyrics {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.response()
    }
}
