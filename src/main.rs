#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod artist;
mod artwork;
mod controls;
mod dynamic_bg;
mod layout;
mod lyrics;
mod title;
mod utils;

use crate::{
    artist::Artist, artwork::Artwork, controls::Controls, dynamic_bg::DynamicBg, layout::Layout,
    lyrics::Lyrics, title::Title, utils::PutClipped,
};
use anyhow::{Context, Result};
use eframe::{
    egui::{self, FontData, FontFamily},
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};

pub const WINDOW_SIZE: [f32; 2] = [1024.0, 600.0];

struct LyricsApp {
    bg: DynamicBg,
    controls: Controls,
    artwork: Artwork,
    lyrics: Lyrics,
    title: Title,
    artist: Artist,
}

impl LyricsApp {
    fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Result<Self> {
        Ok(Self {
            bg: DynamicBg::new(cc).context("failed to make dynamic bg")?,
            controls: Controls::new(),
            artwork: Artwork::new(
                "https://i.scdn.co/image/ab67616d0000b273e65ea6fee33fa7efb58921d1".into(),
            ),
            lyrics: Lyrics::new(),
            title: Title::new("ATTACKING VERTICAL".into()),
            artist: Artist::new(0000, "femtanyl".into()),
        })
    }
}

impl eframe::App for LyricsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if let Some((_, colours, _)) = self.artwork.state() {
            self.bg.set_colours(colours);
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

            Ok(Box::new(LyricsApp::new(cc)?))
        }),
    )
}
