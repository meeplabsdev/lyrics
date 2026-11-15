use crate::{colour::Colour, state::State};
use eframe::egui::{Image, Pos2, Rect, Sense, Stroke, Vec2, Widget, include_image};

#[derive(Debug, Clone, Copy)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

pub struct ArtworkControls {
    state: State,
    playing: bool,
    shuffle: bool,
    repeat: RepeatMode,
}

impl ArtworkControls {
    pub fn new(state: State) -> Self {
        Self {
            state,
            playing: false,
            shuffle: true,
            repeat: RepeatMode::All,
        }
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        self.shuffle = shuffle;
    }

    pub fn set_repeat(&mut self, repeat: RepeatMode) {
        self.repeat = repeat;
    }
}

impl Widget for &mut ArtworkControls {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        let painter = ui.painter();

        // TODO: holy mess
        // and this is duplicated logic from within egui probably anyway
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            if pos.x > rect.left() + (rect.width() / 2.0) - (rect.width() / 20.0)
                && pos.x < rect.left() + (rect.width() / 2.0) + (rect.width() / 20.0)
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.height() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.height() / 20.0)
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
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.height() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.height() / 20.0)
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
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.height() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.height() / 20.0)
            {
                let _ = self.state.next_track();
            }

            if pos.x
                > rect.left() + (rect.width() / 2.0)
                    - (rect.width() / 10.0) * 2.5
                    - (rect.width() / 20.0)
                && pos.x
                    < rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0) * 2.5
                        + (rect.width() / 20.0)
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.height() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.height() / 20.0)
            {
                let _ = self.state.set_shuffle(!self.shuffle);
                self.shuffle = !self.shuffle;
                response.mark_changed();
            }

            if pos.x
                > rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) * 2.5
                    - (rect.width() / 20.0)
                && pos.x
                    < rect.left()
                        + (rect.width() / 2.0)
                        + (rect.width() / 10.0) * 2.5
                        + (rect.width() / 20.0)
                && pos.y > rect.bottom() - (rect.height() / 15.0) - (rect.height() / 20.0)
                && pos.y < rect.bottom() - (rect.height() / 15.0) + (rect.height() / 20.0)
            {
                let new_repeat = match self.repeat {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };

                let _ = self.state.set_repeat(new_repeat);
                self.repeat = new_repeat;
                response.mark_changed();
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
                    Vec2::new(rect.width() / 15.0, rect.width() / 15.0 * 20.0 / 35.0),
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
                    Vec2::new(rect.width() / 15.0, rect.width() / 15.0 * 20.0 / 35.0),
                ),
            );

        if self.shuffle {
            Image::new(include_image!("../../assets/shuffle.svg"))
                .tint(Colour::STRONG)
                .paint_at(
                    ui,
                    Rect::from_center_size(
                        Pos2::new(
                            rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0) * 2.5,
                            rect.bottom() - (rect.height() / 15.0),
                        ),
                        Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 17.0 / 20.0),
                    ),
                );

            painter.circle(
                Pos2::new(
                    rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0) * 2.5,
                    rect.bottom() - (rect.height() / 30.0),
                ),
                rect.width() / 200.0,
                Colour::STRONG,
                Stroke::NONE,
            );
        } else {
            Image::new(include_image!("../../assets/shuffle_off.svg"))
                .tint(Colour::STRONG)
                .paint_at(
                    ui,
                    Rect::from_center_size(
                        Pos2::new(
                            rect.left() + (rect.width() / 2.0) - (rect.width() / 10.0) * 2.5,
                            rect.bottom() - (rect.height() / 15.0),
                        ),
                        Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 17.0 / 20.0),
                    ),
                );
        }

        match self.repeat {
            RepeatMode::Off => {
                Image::new(include_image!("../../assets/repeat_off.svg"))
                    .tint(Colour::STRONG)
                    .paint_at(
                        ui,
                        Rect::from_center_size(
                            Pos2::new(
                                rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) * 2.5,
                                rect.bottom() - (rect.height() / 15.0),
                            ),
                            Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 17.0 / 20.0),
                        ),
                    );
            }
            RepeatMode::All => {
                Image::new(include_image!("../../assets/repeat.svg"))
                    .tint(Colour::STRONG)
                    .paint_at(
                        ui,
                        Rect::from_center_size(
                            Pos2::new(
                                rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) * 2.5,
                                rect.bottom() - (rect.height() / 15.0),
                            ),
                            Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 17.0 / 20.0),
                        ),
                    );

                painter.circle(
                    Pos2::new(
                        rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) * 2.5,
                        rect.bottom() - (rect.height() / 30.0),
                    ),
                    rect.width() / 200.0,
                    Colour::STRONG,
                    Stroke::NONE,
                );
            }
            RepeatMode::One => {
                Image::new(include_image!("../../assets/repeat_one.svg"))
                    .tint(Colour::STRONG)
                    .paint_at(
                        ui,
                        Rect::from_center_size(
                            Pos2::new(
                                rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) * 2.5,
                                rect.bottom() - (rect.height() / 15.0),
                            ),
                            Vec2::new(rect.width() / 20.0, rect.width() / 20.0 * 17.0 / 20.0),
                        ),
                    );

                painter.circle(
                    Pos2::new(
                        rect.left() + (rect.width() / 2.0) + (rect.width() / 10.0) * 2.5,
                        rect.bottom() - (rect.height() / 30.0),
                    ),
                    rect.width() / 200.0,
                    Colour::STRONG,
                    Stroke::NONE,
                );
            }
        }

        response
    }
}
