use crate::logic::spotify::Spotify;
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike};
use librespot_protocol::connect::{Cluster, ClusterUpdate};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum SpotifyEvent {
    ClusterUpdate {
        image_url: String,
        title: String,
        artist: String,
        year: u32,
        length: u32,
        progress: u32,
        playing: bool,
    },
}

pub struct SpotifyState {
    spotify: Arc<Spotify>,
    tx: broadcast::Sender<SpotifyEvent>,
    rx: broadcast::Receiver<SpotifyEvent>,
}

impl SpotifyState {
    pub fn new(spotify: Arc<Spotify>) -> Self {
        let (tx, rx) = broadcast::channel(16);

        Self { spotify, tx, rx }
    }

    pub async fn wait_event(&mut self) -> Result<SpotifyEvent> {
        Ok(self.rx.recv().await?)
    }

    pub async fn set_cluster(&self, cluster: Cluster) -> Result<()> {
        let player_state = cluster.clone().player_state;
        let metadata = &player_state.track.metadata;

        let image_id = metadata
            .get("image_url")
            .and_then(|iu| iu.strip_prefix("spotify:image:"))
            .unwrap_or_default();

        let track_id = player_state
            .track
            .uri
            .strip_prefix("spotify:track:")
            .unwrap_or_default();

        let track = self.spotify.fetch_track(track_id.to_string()).await?;

        let title = track.original_title.clone().unwrap_or_default();

        let artist = track
            .artist
            .iter()
            .filter_map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let year = DateTime::from_timestamp_secs(track.earliest_live_timestamp())
            .map(|d| d.year())
            .unwrap_or(0)
            .max(0) as u32;

        let length = player_state.duration.max(0) as u32;

        let progress = (cluster.server_timestamp_ms - player_state.timestamp
            + player_state.position_as_of_timestamp) as u32;

        let playing = !player_state.is_paused;

        self.tx.send(SpotifyEvent::ClusterUpdate {
            image_url: format!("https://i.scdn.co/image/{image_id}"),
            title,
            artist,
            year,
            length,
            progress,
            playing,
        })?;

        Ok(())
    }

    pub async fn update_cluster(&self, mut update: ClusterUpdate) -> Result<()> {
        // TODO: a better way
        self.set_cluster(update.cluster.take().context("update had no cluster")?)
            .await?;

        Ok(())
    }
}

impl Clone for SpotifyState {
    fn clone(&self) -> Self {
        Self {
            spotify: self.spotify.clone(),
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}
