#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod colour;
mod layout;
mod state;
mod ui;
mod utils;

use crate::{
    layout::Layout,
    state::{Event, State},
    ui::{
        artist::Artist, artwork::Artwork, artwork_controls::ArtworkControls, controls::Controls,
        dynamic_bg::DynamicBg, lyrics::Lyrics, title::Title,
    },
    utils::PutClipped,
};
use anyhow::{Context, Result};
use eframe::{
    egui::{self, FontData, FontFamily},
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};

pub const WINDOW_SIZE: [f32; 2] = [1024.0, 600.0];

struct LyricsApp {
    bg: DynamicBg,
    artwork: Artwork,
    controls: Controls,
    artwork_controls: ArtworkControls,
    title: Title,
    artist: Artist,
    lyrics: Lyrics,
    state: State,
}

impl LyricsApp {
    fn new<'a>(cc: &'a eframe::CreationContext<'a>, state: State) -> Result<Self> {
        Ok(Self {
            bg: DynamicBg::new(cc).context("failed to make dynamic bg")?,
            artwork: Artwork::new(),
            controls: Controls::new(state.clone()),
            artwork_controls: ArtworkControls::new(state.clone()),
            title: Title::new(),
            artist: Artist::new(),
            lyrics: Lyrics::new(),
            state,
        })
    }

    fn handle_event(&mut self, event: Event, ui: &mut egui::Ui) {
        match event {
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

                ui.putc(layout.artwork, &mut self.artwork);
                ui.putc(layout.controls, &mut self.controls);
                ui.putc(layout.artwork_controls, &mut self.artwork_controls);
                ui.putc(layout.title, &mut self.title);
                ui.putc(layout.artist, &mut self.artist);
                ui.putc(layout.lyrics, &mut self.lyrics);
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_resizable(false)
            .with_decorations(false),
        ..Default::default()
    };

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

            let mut state = State::new();
            let app = LyricsApp::new(cc, state.clone())?;

            state
                .set_track("Honestly", "THØRNS, Kasane Teto", 2024, 218)
                .unwrap();

            state
                .set_artwork("https://i.scdn.co/image/ab67616d0000b27340a96de45049b6714636ef2d")
                .unwrap();

            state.set_lyrics(vec![
                String::from("Line 1"),
                String::from("Line 2 but its longer"),
                String::from(
                    "Line 3 and its reeeaaaalllly long WOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO hi",
                ),
                String::from(
                    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                ),
            ]).unwrap();

            state.sync_progress(0.4).unwrap();
            state.sync_playing(true).unwrap();

            Ok(Box::new(app))
        }),
    )
}
