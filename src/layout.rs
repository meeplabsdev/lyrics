use eframe::egui::{Pos2, Rect, Vec2};

pub struct Layout {
    pub qrcode: Rect,
    pub qr_instructions: Rect,
    pub artwork: Rect,
    pub artwork_controls: Rect,
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
        let screen_w = screen.width();
        let center_x = screen_w / 2.0;
        let artwork_scale = screen.width() / 3.2;
        let artwork_offset = (center_x - artwork_scale) / 2.0;

        let qrcode = Rect::from_min_size(
            Pos2::new(center_x - artwork_scale / 2.0, artwork_offset),
            Vec2::new(artwork_scale, artwork_scale),
        );

        let qr_instructions = Rect::from_min_size(
            qrcode.left_bottom() + Vec2::new(-8.0, 8.0),
            Vec2::new(qrcode.width() + 32.0, screen_h / 12.0),
        );

        let artwork = Rect::from_min_size(
            Pos2::new(artwork_offset, artwork_offset),
            Vec2::new(artwork_scale, artwork_scale),
        );

        let artwork_controls = artwork.scale_from_center(0.95);

        let controls = Rect::from_min_size(
            artwork.left_bottom() + Vec2::new(-16.0, 8.0),
            Vec2::new(artwork.width() + 32.0, screen_h / 25.0),
        );

        let title = Rect::from_min_size(
            controls.left_bottom() + Vec2::new(8.0, 4.0),
            Vec2::new(controls.width() - 16.0, screen_h / 20.0),
        );

        let artist = Rect::from_min_size(
            controls.left_bottom() + Vec2::new(8.0, title.height() + 4.0),
            Vec2::new(controls.width() - 16.0, screen_h / 30.0),
        );

        let lyrics = Rect::from_min_max(
            Pos2::new(center_x, 0.0),
            Pos2::new(screen_w * 0.95, screen_h),
        );

        Self {
            qrcode,
            qr_instructions,
            artwork,
            artwork_controls,
            controls,
            title,
            artist,
            lyrics,
        }
    }
}
