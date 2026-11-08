#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dynamic_bg;
mod layout;
mod utils;

use crate::{dynamic_bg::DynamicBg, layout::Layout, utils::PutClipped};
use eframe::egui::{self, Align, Button, Color32, Image, Label, RichText, include_image};

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

                ui.putc(
                    layout.artwork,
                    Image::new(include_image!("../assets/test.png")).corner_radius(4),
                );

                ui.putc(layout.controls, Button::new("hiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii"));

                ui.putc(
                    layout.title,
                    Label::new(
                        RichText::new("Among Us Impostor")
                            .color(Color32::WHITE)
                    )
                    .halign(Align::Center)
                    .selectable(false),
                );

                ui.putc(
                    layout.artist,
                    Label::new(
                        RichText::new("Real Crewmate")
                            .color(Color32::WHITE)
                    )
                    .halign(Align::Center)
                    .selectable(false),
                );

                ui.painter().rect_filled(layout.lyrics, 0, Color32::BLACK);
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
