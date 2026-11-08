use eframe::egui::{Pos2, Rect, Vec2};

pub struct Layout {
    pub artwork: Rect,
    pub controls: Rect,
    pub title: Rect,
    pub artist: Rect,
    pub lyrics: Rect,
}

// This is a really bad way to lay everything out
// Honestly it would probably be better static atp
impl Layout {
    pub fn new(screen: Rect) -> Self {
        let screen_h = screen.height();
        let center_x = screen.center().x;
        let lyrics = Rect::everything_right_of(center_x);

        let artwork_scale = screen.width() / 3.2;
        let artwork_offset = (center_x - artwork_scale) / 2.0;
        let artwork = Rect::from_min_size(
            Pos2::new(artwork_offset, artwork_offset),
            Vec2::new(artwork_scale, artwork_scale),
        );

        let controls = Rect::from_min_size(
            artwork.left_bottom() + Vec2::new(4.0, 8.0),
            Vec2::new(artwork.width() - 8.0, screen_h / 19.0),
        );

        let title = Rect::from_min_size(
            controls.left_bottom() + Vec2::new(8.0, 4.0),
            Vec2::new(controls.width(), screen_h / 25.0),
        );

        let artist = Rect::from_min_size(
            controls.left_bottom() + Vec2::new(8.0, title.height() + 4.0),
            Vec2::new(controls.width(), screen_h / 35.0),
        );

        Self {
            artwork,
            controls,
            title,
            artist,
            lyrics,
        }
    }
}
