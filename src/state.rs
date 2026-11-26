use crate::{colour::Colour, ui::artwork_controls::RepeatMode};
use anyhow::Result;
use color_thief::ColorFormat;
use eframe::egui::ColorImage;
use image::Rgba;
use qrcode::{EcLevel, bits, canvas::Canvas, ec, render::Renderer};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AppState {
    Loading,
    Login,
    Ready,
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    AppStateChanged {
        app_state: AppState,
    },

    // Login
    LoginChanged {
        qr: ColorImage,
        code: String,
    },

    // Ready
    TrackChanged {
        title: String,
        artist: String,
        year: u32,
        length: u32,
    },
    ProgressChanged {
        progress: f32,
    },
    ProgressSync {
        progress: f32,
    },
    ArtworkChanged {
        name: String,
        image: ColorImage,
        colours: Vec<Colour>,
    },
    LyricsChanged {
        lyrics: Vec<String>,
    },
    PlayingChanged {
        playing: bool,
    },
    PlayingSync {
        playing: bool,
    },
    ShuffleChanged {
        shuffle: bool,
    },
    ShuffleSync {
        shuffle: bool,
    },
    RepeatChanged {
        repeat: RepeatMode,
    },
    RepeatSync {
        repeat: RepeatMode,
    },
    NextTrack,
    PreviousTrack,
}

pub struct UIState {
    tx: broadcast::Sender<UIEvent>,
    rx: broadcast::Receiver<UIEvent>,
}

impl UIState {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(16);

        Self { tx, rx }
    }

    pub fn event(&mut self) -> Result<UIEvent> {
        Ok(self.rx.try_recv()?)
    }

    pub async fn wait_event(&mut self) -> Result<UIEvent> {
        Ok(self.rx.recv().await?)
    }

    pub fn set_app_state(&mut self, app_state: AppState) -> Result<()> {
        paris::log!("Set app state to {app_state:?}");
        self.tx.send(UIEvent::AppStateChanged { app_state })?;

        Ok(())
    }

    pub fn set_login(&mut self, code: String) -> Result<()> {
        let bits = bits::encode_auto(
            format!("https://spotify.com/pair?code={code}").as_bytes(),
            EcLevel::H,
        )?;
        let version = bits.version();
        let data = bits.into_bytes();
        let (encoded_data, ec_data) = ec::construct_codewords(&data, version, EcLevel::H)?;

        let mut canvas = Canvas::new(version, EcLevel::H);
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&encoded_data, &ec_data);

        let canvas = canvas.apply_best_mask();
        let qrcode =
            Renderer::<Rgba<u8>>::new(&canvas.into_colors(), version.width() as usize, 1).build();

        let size = [qrcode.width() as usize, qrcode.height() as usize];
        let pixels = qrcode.into_raw();
        let qr = ColorImage::from_rgba_unmultiplied(size, &pixels);

        paris::log!("Set login code to {code}");
        self.tx.send(UIEvent::LoginChanged { qr, code })?;
        self.tx.send(UIEvent::ArtworkChanged {
            name: "QrCode".into(),
            image: ColorImage::from_rgba_unmultiplied([1, 1], &Colour::WEAK.into_array_u8()),
            colours: vec![Colour::WHITE, Colour::BLACK],
        })?;

        Ok(())
    }

    pub fn set_track(
        &mut self,
        title: impl Into<String>,
        artist: impl Into<String>,
        year: u32,
        length: u32,
    ) -> Result<()> {
        let title = title.into();
        let artist = artist.into();

        paris::log!("Set track to {title}");
        self.tx.send(UIEvent::TrackChanged {
            title,
            artist,
            year,
            length,
        })?;

        Ok(())
    }

    pub fn set_progress(&mut self, progress: f32) -> Result<()> {
        paris::log!("Set progress to {progress}");
        self.tx.send(UIEvent::ProgressChanged { progress })?;
        self.tx.send(UIEvent::ProgressSync { progress })?;

        Ok(())
    }

    pub fn sync_progress(&mut self, progress: f32) -> Result<()> {
        paris::log!("Synced progress to {progress}");
        self.tx.send(UIEvent::ProgressSync { progress })?;

        Ok(())
    }

    pub async fn set_artwork(&mut self, uri: impl Into<String>) -> Result<()> {
        let uri = uri.into();

        let resp = reqwest::get(&uri).await?.error_for_status()?;
        let bytes = resp.bytes().await?;

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
            1,
            3,
        )?
        .into_iter()
        .map(Into::into)
        .collect();

        paris::log!("Set artwork to {uri}");
        self.tx.send(UIEvent::ArtworkChanged {
            name: uri,
            image,
            colours,
        })?;

        Ok(())
    }

    pub fn set_lyrics(&mut self, lyrics: Vec<String>) -> Result<()> {
        paris::log!("Set lyrics ({} lines)", lyrics.len());
        self.tx.send(UIEvent::LyricsChanged { lyrics })?;

        Ok(())
    }

    pub fn set_playing(&mut self, playing: bool) -> Result<()> {
        paris::log!("Set playing to {playing}");
        self.tx.send(UIEvent::PlayingChanged { playing })?;
        self.tx.send(UIEvent::PlayingSync { playing })?;

        Ok(())
    }

    pub fn sync_playing(&mut self, playing: bool) -> Result<()> {
        paris::log!("Synced playing to {playing}");
        self.tx.send(UIEvent::PlayingSync { playing })?;

        Ok(())
    }

    pub fn set_shuffle(&mut self, shuffle: bool) -> Result<()> {
        paris::log!("Set shuffle to {shuffle}");
        self.tx.send(UIEvent::ShuffleChanged { shuffle })?;
        self.tx.send(UIEvent::ShuffleSync { shuffle })?;

        Ok(())
    }

    pub fn sync_shuffle(&mut self, shuffle: bool) -> Result<()> {
        paris::log!("Synced shuffle to {shuffle}");
        self.tx.send(UIEvent::ShuffleSync { shuffle })?;

        Ok(())
    }

    pub fn set_repeat(&mut self, repeat: RepeatMode) -> Result<()> {
        paris::log!("Set repeat to {repeat:?}");
        self.tx.send(UIEvent::RepeatChanged { repeat })?;
        self.tx.send(UIEvent::RepeatSync { repeat })?;

        Ok(())
    }

    pub fn sync_repeat(&mut self, repeat: RepeatMode) -> Result<()> {
        paris::log!("Synced repeat to {repeat:?}");
        self.tx.send(UIEvent::RepeatSync { repeat })?;

        Ok(())
    }

    pub fn next_track(&mut self) -> Result<()> {
        paris::log!("Triggered next track");
        self.tx.send(UIEvent::NextTrack)?;

        Ok(())
    }

    pub fn prev_track(&mut self) -> Result<()> {
        paris::log!("Triggered previous track");
        self.tx.send(UIEvent::PreviousTrack)?;

        Ok(())
    }
}

impl Clone for UIState {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}
