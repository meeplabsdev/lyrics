use crate::colour::Colour;
use eframe::egui::{ColorImage, Image, Sense, Shadow, TextureOptions, Widget, load::SizedTexture};

pub struct Artwork {
    name: String,
    image: ColorImage,
}

impl Artwork {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            image: ColorImage::from_rgba_unmultiplied([1, 1], &Colour::WEAK.into_array_u8()),
        }
    }

    pub fn set_image(&mut self, name: String, image: ColorImage) {
        self.name = name;
        self.image = image;
    }
}

impl Widget for &mut Artwork {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), Sense::HOVER | Sense::CLICK);

        let painter = ui.painter();
        let img_rect = rect.scale_from_center(0.96);

        painter.add(
            Shadow {
                blur: (rect.width() - img_rect.width()) as u8,
                color: Colour::from_rgba(0, 0, 0, 100).into(),
                ..Default::default()
            }
            .as_shape(img_rect, 0),
        );

        let texture = ui.ctx().load_texture(
            format!("artwork-{}", self.name),
            self.image.clone(),
            TextureOptions::LINEAR,
        );

        Image::from_texture(SizedTexture::new(texture.id(), img_rect.size()))
            .fit_to_original_size(1.0)
            .corner_radius(6)
            .paint_at(ui, img_rect);

        response
    }
}
