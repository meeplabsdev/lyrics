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
        artist::Artist, artwork::Artwork, controls::Controls, dynamic_bg::DynamicBg,
        lyrics::Lyrics, title::Title,
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
    state: State,
    bg: DynamicBg,
    controls: Controls,
    artwork: Artwork,
    lyrics: Lyrics,
    title: Title,
    artist: Artist,
}

impl LyricsApp {
    fn new<'a>(cc: &'a eframe::CreationContext<'a>, state: State) -> Result<Self> {
        Ok(Self {
            bg: DynamicBg::new(cc).context("failed to make dynamic bg")?,
            controls: Controls::new(state.clone()),
            artwork: Artwork::new(),
            lyrics: Lyrics::new(),
            title: Title::new(),
            artist: Artist::new(),
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
                self.title.set_title(title);
            }
            Event::ArtworkChanged {
                name,
                image,
                colours,
            } => {
                self.artwork.set_image(name, image);
                self.bg.set_colours(colours);
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
                .set_track(
                    "NTR Ending (Cardiac Arrest Due to Overdose)",
                    "SWRSLT",
                    0000,
                    115,
                )
                .unwrap();

            state.set_progress(0.4).unwrap();

            state
                .set_artwork("https://i.scdn.co/image/ab67616d0000b27320754a1128d65c6b9688ea3d")
                .unwrap();

            Ok(Box::new(app))
        }),
    )
}
