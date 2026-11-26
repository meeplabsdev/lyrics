use anyhow::Result;
use librespot_protocol::connect::{Cluster, ClusterUpdate};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum SpotifyEvent {
    Update { image_url: String, title: String },
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
        // TODO: take properties and fire events

        paris::log!("Got cluster {:#?}", cluster);
        self.cluster = Some(cluster);

        Ok(())
    }

    pub async fn update_cluster(&mut self, update: ClusterUpdate) -> Result<()> {
        // TODO
        paris::log!("{update:?}");

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
