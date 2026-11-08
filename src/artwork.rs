use anyhow::Result;
use color_thief::ColorFormat;
use eframe::egui::{Color32, ColorImage, Context, Image, TextureHandle, TextureOptions, Widget};
use rgb::Rgb;
use std::sync::{Arc, Mutex};

enum State {
    Loading,
    Ready((ColorImage, Vec<Rgb<u8>>)),
    Failed(String),
}

pub struct Artwork {
    uri: String,
    state: Arc<Mutex<State>>,
}

impl Artwork {
    pub fn new(uri: String) -> Self {
        let state = Arc::new(Mutex::new(State::Loading));

        let fetch_state = Arc::clone(&state);
        let fetch_uri = uri.clone();

        std::thread::spawn(move || {
            let result = fetch(&fetch_uri).and_then(|image| {
                Ok((
                    image.clone(),
                    color_thief::get_palette(
                        &image
                            .pixels
                            .iter()
                            .flat_map(|a| a.to_array())
                            .collect::<Vec<u8>>(),
                        ColorFormat::Rgba,
                        4,
                        4,
                    )?,
                ))
            });

            let mut state = fetch_state.lock().unwrap();
            *state = match result {
                Ok(result) => State::Ready(result),
                Err(error) => State::Failed(error.to_string()),
            };
        });

        Self { uri, state }
    }

    pub fn state(&self) -> Option<(Vec<Color32>, Vec<Rgb<u8>>, [usize; 2])> {
        let state = self.state.lock().unwrap();

        match &*state {
            State::Ready((image, colours)) => Some((
                image.pixels.clone(),
                colours.clone(),
                [image.width(), image.height()],
            )),
            _ => None,
        }
    }
}

impl Widget for &mut Artwork {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let state = self.state.lock().unwrap();

        match &*state {
            State::Loading => {
                ui.spinner();
                ui.ctx().request_repaint();
                ui.response()
            }
            State::Failed(err) => ui.label(format!("Failed: {err}")),
            State::Ready((image, _)) => ui.add(
                Image::from_texture(&texture(ui.ctx(), &self.uri, image))
                    .fit_to_original_size(1.0)
                    .max_size(ui.available_size())
                    .corner_radius(4),
            ),
        }
    }
}

fn fetch(uri: &str) -> Result<ColorImage> {
    let resp = reqwest::blocking::get(uri)?.error_for_status()?;
    let bytes = resp.bytes()?;

    let image = image::load_from_memory(&bytes)?;
    let rgba = image.to_rgba8();

    let size = [image.width() as usize, image.height() as usize];
    let pixels = rgba.into_raw();

    Ok(ColorImage::from_rgba_unmultiplied(size, &pixels))
}

fn texture(ctx: &Context, uri: &str, image: &ColorImage) -> TextureHandle {
    ctx.load_texture(
        format!("artwork-{uri}"),
        image.clone(),
        TextureOptions::LINEAR,
    )
}
