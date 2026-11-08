use eframe::egui::{Color32, Pos2, Rect, Sense, Slider, Stroke, Vec2, Widget, style::HandleShape};

pub struct Controls {}

impl Controls {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Controls {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 6.0),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let progress = 0.4;
        let handle_x = rect.left() + rect.width() * progress;

        painter.rect_filled(
            rect,
            6.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 40), // future audio
        );

        painter.rect_filled(
            Rect::from_min_max(rect.min, Pos2::new(handle_x, rect.max.y)),
            6.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 200), // past audio
        );

        painter.circle_filled(
            Pos2::new(handle_x, rect.center().y),
            7.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 100), // handle
        );

        response
    }
}
