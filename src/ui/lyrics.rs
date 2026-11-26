use crate::{colour::Colour, playback::Playback};
use eframe::egui::{FontFamily, FontId, Pos2, Sense, Widget};

pub struct Lyrics {
    lyrics: Vec<String>,
    playback: Playback,
}

impl Lyrics {
    pub fn new(playback: Playback) -> Self {
        Self {
            lyrics: Vec::new(),
            playback,
        }
    }

    pub fn set_lyrics(&mut self, lyrics: Vec<String>) {
        self.lyrics = lyrics;
    }

    // TODO: obviously use line timings, this is fine for now
    fn is_focussed(&self, idx: usize) -> bool {
        self.playback.progress_f32() >= idx as f32 / self.lyrics.len() as f32
            && self.playback.progress_f32() < (idx + 1) as f32 / self.lyrics.len() as f32
    }

    fn center_idx(&self) -> usize {
        let len = self.lyrics.len();
        if len == 0 {
            return 0;
        }

        let idx = (self.playback.progress_f32() * len as f32) as usize;
        idx.min(len - 1)
    }
}

impl Widget for &mut Lyrics {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::empty());
        let painter = ui.painter();

        if self.lyrics.is_empty() {
            return response;
        }

        let center = self.center_idx();
        let font_id = FontId::new(rect.height() / 15.0, FontFamily::Name("lyricsbold".into()));
        let galleys = self
            .lyrics
            .clone()
            .into_iter()
            .map(|line| {
                painter.layout(line, font_id.clone(), Colour::MODERATE.into(), rect.width())
            })
            .collect::<Vec<_>>();

        let height_before = galleys.iter().take(center).map(|g| g.size().y).sum::<f32>();
        let center_height = galleys[center].size().y;

        let mut line_pos = Pos2::new(
            rect.left(),
            (rect.top() + rect.height() * 0.42) - center_height * 0.5 - height_before,
        );
        for (idx, mut galley) in galleys.into_iter().enumerate() {
            if self.is_focussed(idx) {
                galley = ui.painter().layout(
                    galley.text().to_string(),
                    font_id.clone(),
                    Colour::STRONG.into(),
                    rect.width(),
                );
            }

            painter.galley(line_pos, galley.clone(), Colour::MODERATE.into());

            line_pos.y += galley.size().y;
        }

        response
    }
}
