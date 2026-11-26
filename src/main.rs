#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod colour;
mod layout;
mod logic;
mod playback;
mod state;
mod ui;
mod utils;

use std::sync::{Arc, Mutex};

use crate::{
    colour::Colour,
    layout::Layout,
    logic::Logic,
    playback::Playback,
    state::{AppState, UIEvent, UIState},
    ui::{
        artist::Artist, artwork::Artwork, artwork_controls::ArtworkControls, controls::Controls,
        dynamic_bg::DynamicBg, lyrics::Lyrics, qr_instructions::QrInstructions, qrcode::QrCode,
        title::Title,
    },
    utils::PutClipped,
};
use anyhow::{Context, Result};
use eframe::{
    egui::{self, FontData, FontFamily, Spinner},
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};

pub const WINDOW_SIZE: [f32; 2] = [1024.0, 600.0];

struct Inner {
    bg: DynamicBg,
    qrcode: QrCode,
    qr_instructions: QrInstructions,
    artwork: Artwork,
    controls: Controls,
    artwork_controls: ArtworkControls,
    title: Title,
    artist: Artist,
    lyrics: Lyrics,

    app_state: AppState,
    // state: UIState,
    playback: Playback,
}

struct LyricsApp {
    inner: Arc<Mutex<Inner>>,
    logic: JoinHandle<Result<()>>,
}

impl LyricsApp {
    fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        state: UIState,
        logic: Logic,
        handle: &Handle,
    ) -> Result<Self> {
        let playback = Playback::new();
        let logic = logic.spawn(handle);
        let mut inner = Inner {
            bg: DynamicBg::new(cc).context("failed to make dynamic bg")?,
            qrcode: QrCode::new(),
            qr_instructions: QrInstructions::new(),
            artwork: Artwork::new(),
            controls: Controls::new(state.clone(), playback.clone()),
            artwork_controls: ArtworkControls::new(state.clone()),
            title: Title::new(),
            artist: Artist::new(),
            lyrics: Lyrics::new(playback.clone()),

            app_state: AppState::Loading,
            // state: state.clone(),
            playback,
        };

        inner.bg.set_colours(vec![
            Colour::from_rgb(30, 30, 30),
            Colour::from_rgb(10, 10, 10),
        ]);

        let inner = Arc::new(Mutex::new(inner));

        let ctx = cc.egui_ctx.clone();
        let task_inner = inner.clone();
        let mut task_state = state;
        handle.spawn(async move {
            loop {
                let event = task_state.wait_event().await;
                let res = event.and_then(|e| task_inner.lock().unwrap().handle_event(e));
                if let Err(err) = res {
                    paris::error!("{err}");
                }

                ctx.request_repaint();
            }
        });

        Ok(Self { inner, logic })
    }
}

impl Inner {
    fn handle_event(&mut self, event: UIEvent) -> Result<()> {
        match event {
            UIEvent::AppStateChanged { app_state } => {
                self.app_state = app_state;
            }
            UIEvent::LoginChanged { qr, code } => {
                self.qrcode.set_image(qr);
                self.qr_instructions.set_code(code);
            }
            UIEvent::TrackChanged {
                title,
                artist,
                year,
                length,
            } => {
                self.artist.set_artist(artist, year);
                self.playback.set_length(length)?;
                self.title.set_title(title);
            }
            UIEvent::ProgressSync { progress } => {
                self.playback.set_progress(progress)?;
            }
            UIEvent::ArtworkChanged {
                name,
                image,
                colours,
            } => {
                self.artwork.set_image(name, image);
                self.bg.set_colours(colours);
            }
            UIEvent::LyricsChanged { lyrics } => {
                self.lyrics.set_lyrics(lyrics);
            }
            UIEvent::PlayingSync { playing } => {
                self.artwork_controls.set_playing(playing);
                self.playback.set_playing(playing)?;
            }
            UIEvent::ShuffleSync { shuffle } => {
                self.artwork_controls.set_shuffle(shuffle);
            }
            UIEvent::RepeatSync { repeat } => {
                self.artwork_controls.set_repeat(repeat);
            }
            _ => {}
        }

        Ok(())
    }
}

impl eframe::App for LyricsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let mut guard = self.inner.lock().unwrap();
        let this = &mut *guard;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let layout = Layout::new(ui.available_rect_before_wrap());
                ui.add(&mut this.bg);

                match this.app_state {
                    AppState::Loading => {
                        Spinner::new().paint_at(ui, layout.spinner);
                    }
                    AppState::Login => {
                        ui.putc(layout.qrcode, &mut this.qrcode);
                        ui.putc(layout.qr_instructions, &mut this.qr_instructions);
                    }
                    AppState::Ready => {
                        ui.putc(layout.artwork, &mut this.artwork);
                        ui.putc(layout.controls, &mut this.controls);
                        ui.putc(layout.artwork_controls, &mut this.artwork_controls);
                        ui.putc(layout.title, &mut this.title);
                        ui.putc(layout.artist, &mut this.artist);
                        ui.putc(layout.lyrics, &mut this.lyrics);
                    }
                }
            });
    }

    fn on_exit(&mut self) {
        self.logic.abort();
    }
}

fn main() -> eframe::Result {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_resizable(false)
            .with_decorations(false),
        ..Default::default()
    };

    let state = UIState::new();
    let runtime = Runtime::new().unwrap();
    let logic = Logic::new(state.clone());

    eframe::run_native(
        "Lyrics",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            cc.egui_ctx.add_font(FontInsert::new(
                "lyricsregular",
                FontData::from_static(include_bytes!("../assets/LyricsRegular.ttf")),
                vec![InsertFontFamily {
                    family: FontFamily::Name("lyricsregular".into()),
                    priority: FontPriority::Highest,
                }],
            ));

            cc.egui_ctx.add_font(FontInsert::new(
                "lyricsmedium",
                FontData::from_static(include_bytes!("../assets/LyricsMedium.ttf")),
                vec![InsertFontFamily {
                    family: FontFamily::Name("lyricsmedium".into()),
                    priority: FontPriority::Highest,
                }],
            ));

            cc.egui_ctx.add_font(FontInsert::new(
                "lyricssemibold",
                FontData::from_static(include_bytes!("../assets/LyricsSemibold.ttf")),
                vec![InsertFontFamily {
                    family: FontFamily::Name("lyricssemibold".into()),
                    priority: FontPriority::Highest,
                }],
            ));

            cc.egui_ctx.add_font(FontInsert::new(
                "lyricsbold",
                FontData::from_static(include_bytes!("../assets/LyricsBold.ttf")),
                vec![InsertFontFamily {
                    family: FontFamily::Name("lyricsbold".into()),
                    priority: FontPriority::Highest,
                }],
            ));

            let app = LyricsApp::new(cc, state.clone(), logic, runtime.handle())?;

            Ok(Box::new(app))
        }),
    )
}
