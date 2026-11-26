use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

struct Inner {
    anchor: DateTime<Utc>,
    anchor_ms: u32,
    length_ms: u32,
    playing: bool,
}

#[derive(Clone)]
pub struct Playback(Arc<Mutex<Inner>>);

impl Playback {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner {
            anchor: Utc::now(),
            anchor_ms: 0,
            length_ms: 0,
            playing: false,
        })))
    }

    pub fn reset(&mut self, progress_ms: u32, length_ms: u32, playing: bool) -> Result<()> {
        let mut s = self.0.lock().map_err(|_| anyhow!("failed to lock inner"))?;
        *s = Inner {
            anchor: Utc::now(),
            anchor_ms: progress_ms,
            length_ms,
            playing,
        };

        Ok(())
    }

    pub fn set_progress(&mut self, progress_ms: u32) -> Result<()> {
        let mut s = self.0.lock().map_err(|_| anyhow!("failed to lock inner"))?;
        s.anchor = Utc::now();
        s.anchor_ms = progress_ms;

        Ok(())
    }

    pub fn set_length(&mut self, length_ms: u32) -> Result<()> {
        let mut s = self.0.lock().map_err(|_| anyhow!("failed to lock inner"))?;
        s.length_ms = length_ms;

        Ok(())
    }

    pub fn set_playing(&mut self, playing: bool) -> Result<()> {
        let mut s = self.0.lock().map_err(|_| anyhow!("failed to lock inner"))?;
        s.playing = playing;

        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.lock().map(|s| s.length_ms).unwrap_or(1)
    }

    pub fn progress(&self) -> u32 {
        self.0
            .lock()
            .map(|s| {
                if s.playing {
                    (s.anchor_ms
                        + Utc::now()
                            .signed_duration_since(s.anchor)
                            .num_milliseconds() as u32)
                        .min(s.length_ms)
                } else {
                    s.anchor_ms
                }
            })
            .unwrap_or(0)
    }

    pub fn progress_f32(&self) -> f32 {
        let progress = self.progress();
        let length = self.length();

        progress as f32 / length as f32
    }
}
