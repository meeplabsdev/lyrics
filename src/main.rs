#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod colour;
mod layout;
mod logic;
mod state;
mod ui;
mod utils;

use crate::{
    colour::Colour,
    layout::Layout,
    logic::Logic,
    state::{AppState, Event, State},
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
use tokio::{runtime::Runtime, task::JoinHandle};

pub const WINDOW_SIZE: [f32; 2] = [1024.0, 600.0];

struct LyricsApp {
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
    state: State,
    logic: JoinHandle<Result<()>>,
}

impl LyricsApp {
    fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        state: State,
        logic: JoinHandle<Result<()>>,
    ) -> Result<Self> {
        let mut this = Self {
            bg: DynamicBg::new(cc).context("failed to make dynamic bg")?,
            qrcode: QrCode::new(),
            qr_instructions: QrInstructions::new(),
            artwork: Artwork::new(),
            controls: Controls::new(state.clone()),
            artwork_controls: ArtworkControls::new(state.clone()),
            title: Title::new(),
            artist: Artist::new(),
            lyrics: Lyrics::new(),

            app_state: AppState::Loading,
            state,
            logic,
        };

        this.bg.set_colours(vec![
            Colour::from_rgb(30, 30, 30),
            Colour::from_rgb(10, 10, 10),
        ]);

        Ok(this)
    }

    fn handle_event(&mut self, event: Event, ui: &mut egui::Ui) {
        match event {
            Event::AppStateChanged { app_state } => {
                self.app_state = app_state;
            }
            Event::LoginChanged { qr, code } => {
                self.qrcode.set_image(qr);
                self.qr_instructions.set_code(code);
            }
            Event::TrackChanged {
                title,
                artist,
                year,
                length,
            } => {
                self.artist.set_artist(artist, year);
                self.controls.set_length(length);
                self.controls.set_progress(0.0);
                self.lyrics.set_progress(0.0);
                self.title.set_title(title);
            }
            Event::ProgressSync { progress } => {
                self.controls.set_progress(progress);
                self.lyrics.set_progress(progress);
            }
            Event::ArtworkChanged {
                name,
                image,
                colours,
            } => {
                self.artwork.set_image(name, image);
                self.bg.set_colours(colours);
            }
            Event::LyricsChanged { lyrics } => {
                self.lyrics.set_lyrics(lyrics);
            }
            Event::PlayingSync { playing } => {
                self.artwork_controls.set_playing(playing);
            }
            Event::ShuffleSync { shuffle } => {
                self.artwork_controls.set_shuffle(shuffle);
            }
            Event::RepeatSync { repeat } => {
                self.artwork_controls.set_repeat(repeat);
            }
            _ => {}
        }

        ui.request_repaint();
    }
}

impl eframe::App for LyricsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        loop {
            match self.state.event() {
                Ok(event) => self.handle_event(event, ui),
                Err(_) => break,
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let layout = Layout::new(ui.available_rect_before_wrap());
                ui.add(&mut self.bg);

                match self.app_state {
                    AppState::Loading => {
                        Spinner::new().paint_at(ui, layout.spinner);
                    }
                    AppState::Login => {
                        ui.putc(layout.qrcode, &mut self.qrcode);
                        ui.putc(layout.qr_instructions, &mut self.qr_instructions);
                    }
                    AppState::Ready => {
                        ui.putc(layout.artwork, &mut self.artwork);
                        ui.putc(layout.controls, &mut self.controls);
                        ui.putc(layout.artwork_controls, &mut self.artwork_controls);
                        ui.putc(layout.title, &mut self.title);
                        ui.putc(layout.artist, &mut self.artist);
                        ui.putc(layout.lyrics, &mut self.lyrics);
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

    let state = State::new();
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

            let app_state = state.clone();
            let handle = logic.spawn(runtime.handle());
            let app = LyricsApp::new(cc, app_state, handle)?;

            Ok(Box::new(app))
        }),
    )
}
