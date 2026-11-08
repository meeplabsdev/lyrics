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
use eframe::egui;

pub const WINDOW_SIZE: [f32; 2] = [1024.0, 600.0];

struct LyricsApp {
    bg: DynamicBg,
}

impl LyricsApp {
    fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        Some(Self {
            bg: DynamicBg::new(cc)?,
        })
    }
}

impl eframe::App for LyricsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let layout = Layout::new(ui.available_rect_before_wrap());
                ui.add(&mut self.bg);

                ui.putc(layout.artwork, Artwork::new());
                ui.putc(layout.controls, Controls::new());
                ui.putc(layout.title, Title::new());
                ui.putc(layout.artist, Artist::new());
                ui.putc(layout.lyrics, Lyrics::new());
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

            Ok(Box::new(LyricsApp::new(cc).ok_or("err")?))
        }),
    )
}
