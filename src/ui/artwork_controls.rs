use crate::{colour::Colour, state::State};
use eframe::egui::{Image, Pos2, Rect, Sense, Vec2, Widget, include_image};

pub struct ArtworkControls {
    state: State,
    playing: bool,
}

impl ArtworkControls {
    pub fn new(state: State) -> Self {
        Self {
            state,
            playing: false,
        }
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }
}

impl Widget for &mut ArtworkControls {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        // TODO: holy mess
        // and this is duplicated logic from within egui probably anyway
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            if pos.x > rect.left() + (rect.width() / 2.0) - (rect.width() / 20.0)
                && pos.x < rect.left() + (rect.width() / 2.0) + (rect.width() / 20.0)
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.width() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.width() / 20.0)
            {
                let _ = self.state.set_playing(!self.playing);
                self.playing = !self.playing;
                response.mark_changed();
            }

            if pos.x
                > rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0) - (rect.width() / 20.0)
                && pos.x
                    < rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0)
                        + (rect.width() / 20.0)
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.width() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.width() / 20.0)
            {
                let _ = self.state.prev_track();
            }

            if pos.x
                > rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) - (rect.width() / 20.0)
                && pos.x
                    < rect.left()
                        + (rect.width() / 2.0)
                        + (rect.width() / 10.0)
                        + (rect.width() / 20.0)
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.width() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.width() / 20.0)
            {
                let _ = self.state.next_track();
            }
        }

        if self.playing {
            Image::new(include_image!("../../assets/pause.svg"))
                .tint(Colour::STRONG)
                .paint_at(
                    ui,
                    Rect::from_center_size(
                        Pos2::new(
                            rect.left() + (rect.width() / 2.0),
                            rect.bottom() - (rect.height() / 15.0),
                        ),
                        Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 20.0 / 15.0),
                    ),
                );
        } else {
            Image::new(include_image!("../../assets/play.svg"))
                .tint(Colour::STRONG)
                .paint_at(
                    ui,
                    Rect::from_center_size(
                        Pos2::new(
                            rect.left() + (rect.width() / 2.0),
                            rect.bottom() - (rect.height() / 15.0),
                        ),
                        Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 20.0 / 18.0),
                    ),
                );
        }

        Image::new(include_image!("../../assets/prev.svg"))
            .tint(Colour::STRONG)
            .paint_at(
                ui,
                Rect::from_center_size(
                    Pos2::new(
                        rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0),
                        rect.bottom() - (rect.height() / 15.0),
                    ),
                    Vec2::new(rect.height() / 15.0, rect.height() / 15.0 * 20.0 / 35.0),
                ),
            );

        Image::new(include_image!("../../assets/next.svg"))
            .tint(Colour::STRONG)
            .paint_at(
                ui,
                Rect::from_center_size(
                    Pos2::new(
                        rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0),
                        rect.bottom() - (rect.height() / 15.0),
                    ),
                    Vec2::new(rect.height() / 15.0, rect.height() / 15.0 * 20.0 / 35.0),
                ),
            );

        response
    }
}
