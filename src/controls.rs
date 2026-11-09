use crate::colour::Colour;
use eframe::egui::{Align2, FontFamily, FontId, Pos2, Rect, Sense, Vec2, Widget};

pub struct Controls {
    progress: f32,
    length: i32,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            progress: 0.4,
            length: 115,
        }
    }
}

impl Widget for &mut Controls {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (rect, mut response) =
            ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        let offset_h = 42.0;
        let track_start = offset_h + rect.left();
        let track_width = (rect.width() - 2.0 * offset_h).max(0.0);

        if track_width > 0.0 {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let relative_x = pointer_pos.x - track_start;
                let new_progress = (relative_x / track_width).clamp(0.0, 1.0);

                if (new_progress - self.progress).abs() > f32::EPSILON {
                    self.progress = new_progress;
                    response.mark_changed();
                }
            }
        }

        let painter = ui.painter_at(rect);
        let handle_x = track_start + (rect.width() - 2.0 * offset_h) * self.progress;

        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(handle_x, rect.center().y - 2.0),
                Vec2::new(track_width * (1.0 - self.progress), 4.0),
            ),
            6.0,
            Colour::WEAK,
        );

        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(track_start, rect.center().y - 2.0),
                Vec2::new(track_width * self.progress, 4.0),
            ),
            6.0,
            Colour::STRONG,
        );

        painter.circle_filled(
            Pos2::new(handle_x, rect.center().y),
            4.2,
            Colour::WHITE, // handle
        );

        painter.text(
            Pos2::new(offset_h - 8.0 + rect.left(), rect.center().y),
            Align2::RIGHT_CENTER,
            seconds_to_string((self.length as f32 * self.progress) as i32),
            FontId::new(12.0, FontFamily::Name("lyricsmedium".into())),
            Colour::WHITE.into(),
        );

        painter.text(
            Pos2::new(8.0 - offset_h + rect.right(), rect.center().y),
            Align2::LEFT_CENTER,
            seconds_to_string(self.length),
            FontId::new(12.0, FontFamily::Name("lyricsmedium".into())),
            Colour::WHITE.into(),
        );

        ui.request_repaint_after_secs(0.1);

        response
    }
}

fn seconds_to_string(seconds: i32) -> String {
    let mins = seconds / 60;
    let rem = seconds % 60;

    format!("{mins:02}:{rem:02}")
}
