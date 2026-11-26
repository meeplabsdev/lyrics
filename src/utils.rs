use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use eframe::egui::{Rect, Response, Ui, Widget};

pub trait PutClipped {
    fn putc(&mut self, rect: Rect, widget: impl Widget) -> Response;
}

impl PutClipped for Ui {
    fn putc(&mut self, rect: Rect, widget: impl Widget) -> Response {
        let clip = rect.intersect(self.clip_rect());

        self.allocate_ui(rect.size(), |child| {
            child.set_clip_rect(clip);
            child.put(rect, widget);
        })
        .response
    }
}

pub enum ExpiringOption<T> {
    None,
    Some { inner: T, expires: Instant },
}

#[allow(dead_code)]
impl<T> ExpiringOption<T> {
    pub fn none() -> Self {
        Self::None
    }

    pub fn some_with_exiry(inner: T, expires: Instant) -> Self {
        Self::Some { inner, expires }
    }

    pub fn some_with_life(inner: T, life: Duration) -> Self {
        Self::Some {
            inner,
            expires: Instant::now() + life,
        }
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            Self::None => None,
            Self::Some { inner, expires } => {
                if *expires < Instant::now() {
                    return None;
                }

                Some(inner)
            }
        }
    }
}

pub fn spotify_id_to_gid(id: &str) -> Result<String> {
    let mut value = 0u128;
    for byte in id.as_bytes() {
        let digit = match byte {
            b'0'..=b'9' => (byte - b'0') as u128,
            b'a'..=b'z' => (byte - b'a' + 10) as u128,
            b'A'..=b'Z' => (byte - b'A' + 36) as u128,
            _ => return Err(anyhow!("invalid spotify id")),
        };

        value *= 62;
        value += digit;
    }

    Ok(format!("{value:032x}"))
}
