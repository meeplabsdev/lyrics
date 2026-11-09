use eframe::egui::Color32;
use rgb::{Rgb, Rgba};

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Colour {
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);

    pub const STRONG: Self = Self::from_rgba(255, 255, 255, 200);
    pub const WEAK: Self = Self::from_rgba(255, 255, 255, 40);

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn into_array_u8(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn into_array_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

impl From<Color32> for Colour {
    fn from(value: Color32) -> Self {
        Self {
            r: value.r(),
            g: value.g(),
            b: value.b(),
            a: value.a(),
        }
    }
}

impl From<Rgba<u8>> for Colour {
    fn from(value: Rgba<u8>) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl From<Rgb<u8>> for Colour {
    fn from(value: Rgb<u8>) -> Self {
        value.with_alpha(255).into()
    }
}

impl From<Colour> for Color32 {
    fn from(value: Colour) -> Self {
        Self::from_rgba_unmultiplied(value.r, value.g, value.b, value.a)
    }
}

impl From<Colour> for Rgba<u8> {
    fn from(value: Colour) -> Self {
        Self::new(value.r, value.g, value.b, value.a)
    }
}

impl From<Colour> for Rgb<u8> {
    fn from(value: Colour) -> Self {
        Self::new(value.r, value.g, value.b)
    }
}
