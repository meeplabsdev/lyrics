use eframe::egui::Widget;

pub struct Controls {}

impl Controls {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Controls {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.response()
    }
}
