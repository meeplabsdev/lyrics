use anyhow::{Context, Result};
use librespot_protocol::connect::{Cluster, ClusterUpdate};
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
    cluster: Option<Cluster>,
    tx: broadcast::Sender<SpotifyEvent>,
    rx: broadcast::Receiver<SpotifyEvent>,
}

impl SpotifyState {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(16);

        Self {
            cluster: None,
            tx,
            rx,
        }
    }

    // pub fn event(&mut self) -> Result<SpotifyEvent> {
    //     Ok(self.rx.try_recv()?)
    // }

    pub async fn wait_event(&mut self) -> Result<SpotifyEvent> {
        Ok(self.rx.recv().await?)
    }

    pub async fn set_cluster(&mut self, cluster: Cluster) -> Result<()> {
        let player_state = cluster.clone().player_state;
        let metadata = &player_state.track.metadata;

        let image_id = metadata
            .get("image_url")
            .and_then(|iu| iu.strip_prefix("spotify:image:"))
            .unwrap_or_default();

        let title = metadata.get("title").cloned().unwrap_or_default();
        let artist = metadata
            .get("artist_uri")
            .and_then(|iu| iu.strip_prefix("spotify:artist:"))
            .unwrap_or_default();

        let length = player_state.duration.max(0) as u32;
        let progress = (cluster.server_timestamp_ms - player_state.timestamp
            + player_state.position_as_of_timestamp) as u32;

        let playing = !player_state.is_paused;

        self.tx.send(SpotifyEvent::ClusterUpdate {
            image_url: format!("https://i.scdn.co/image/{image_id}"),
            title,
            artist: artist.to_string(),
            year: 0,
            length,
            progress,
            playing,
        })?;

        // paris::log!("Got cluster {:#?}", cluster);
        self.cluster = Some(cluster);

        Ok(())
    }

    pub async fn update_cluster(&mut self, mut update: ClusterUpdate) -> Result<()> {
        // TODO: a better way
        self.set_cluster(update.cluster.take().context("update had no cluster")?)
            .await?;

        Ok(())
    }
}

impl Clone for SpotifyState {
    fn clone(&self) -> Self {
        Self {
            cluster: self.cluster.clone(),
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}
