use crate::colour::Colour;
use anyhow::Result;
use color_thief::ColorFormat;
use eframe::egui::ColorImage;
use tokio::sync::broadcast::{self, error::SendError};

// TODO: PlayingChanged
#[derive(Debug, Clone)]
pub enum Event {
    TrackChanged {
        title: String,
        artist: String,
        year: u32,
        length: u32,
    },
    ProgressChanged {
        progress: f32,
    },
    ArtworkChanged {
        name: String,
        image: ColorImage,
        colours: Vec<Colour>,
    },
}

pub struct State {
    tx: broadcast::Sender<Event>,
    rx: broadcast::Receiver<Event>,
}

impl State {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(16);

        Self { tx, rx }
    }

    pub fn event(&mut self) -> Result<Event, broadcast::error::TryRecvError> {
        self.rx.try_recv()
    }

    pub fn set_track(
        &mut self,
        title: impl Into<String>,
        artist: impl Into<String>,
        year: u32,
        length: u32,
    ) -> Result<(), SendError<Event>> {
        self.tx.send(Event::TrackChanged {
            title: title.into(),
            artist: artist.into(),
            year,
            length,
        })?;

        Ok(())
    }

    pub fn set_progress(&mut self, progress: f32) -> Result<(), SendError<Event>> {
        self.tx.send(Event::ProgressChanged { progress })?;

        Ok(())
    }

    pub fn set_artwork(&mut self, uri: impl Into<String>) -> Result<()> {
        let uri = uri.into();
        let resp = reqwest::blocking::get(&uri)?.error_for_status()?;
        let bytes = resp.bytes()?;

        let image = image::load_from_memory(&bytes)?;
        let rgba = image.to_rgba8();

        let size = [image.width() as usize, image.height() as usize];
        let pixels = rgba.into_raw();
        let image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        let colours = color_thief::get_palette(
            &image
                .pixels
                .iter()
                .flat_map(|a| a.to_array())
                .collect::<Vec<u8>>(),
            ColorFormat::Rgba,
            4,
            4,
        )?
        .into_iter()
        .map(Into::into)
        .collect();

        self.tx.send(Event::ArtworkChanged {
            name: uri,
            image,
            colours,
        })?;

        Ok(())
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}
