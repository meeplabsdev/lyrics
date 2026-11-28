use crate::{colour::Colour, playback::Playback, state::UIState};
use eframe::egui::{Align2, FontFamily, FontId, Pos2, Rect, Sense, Vec2, Widget};

pub struct Controls {
    state: UIState,
    playback: Playback,
    scrub_progress: Option<f32>,
}

impl Controls {
    pub fn new(state: UIState, playback: Playback) -> Self {
        Self {
            state,
            playback,
            scrub_progress: None,
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
                let new_progress = ((pointer_pos.x - track_start) / track_width).clamp(0.0, 1.0);

                if new_progress - self.playback.progress_f32() > f32::EPSILON {
                    self.scrub_progress = Some(new_progress);
                    response.mark_changed();
                }
            } else if let Some(scrub_progress) = self.scrub_progress.take() {
                let _ = self
                    .state
                    .set_progress((scrub_progress * self.playback.length() as f32) as u32);
                // self.progress = scrub_progress;
                response.mark_changed();
            }
        }

        let painter = ui.painter_at(rect);
        let progress = self.scrub_progress.unwrap_or(self.playback.progress_f32());
        let handle_x = track_start + (rect.width() - 2.0 * offset_h) * progress;

        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(handle_x, rect.center().y - 2.0),
                Vec2::new(track_width * (1.0 - progress), 4.0),
            ),
            6.0,
            Colour::WEAK,
        );

        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(track_start, rect.center().y - 2.0),
                Vec2::new(track_width * progress, 4.0),
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
            ms_to_string((self.playback.length() as f32 * progress) as u32),
            FontId::new(12.0, FontFamily::Name("lyricsmedium".into())),
            Colour::WHITE.into(),
        );

        painter.text(
            Pos2::new(8.0 - offset_h + rect.right(), rect.center().y),
            Align2::LEFT_CENTER,
            ms_to_string(self.playback.length()),
            FontId::new(12.0, FontFamily::Name("lyricsmedium".into())),
            Colour::WHITE.into(),
        );

        ui.request_repaint_after_secs(0.1);

        response
    }
}

fn ms_to_string(millis: u32) -> String {
    let seconds = millis / 1000;
    let mins = seconds / 60;
    let rem = seconds % 60;

    format!("{mins:02}:{rem:02}")
}
