mod access_token;
mod apresolve;
mod device_id;
mod device_login;
mod spotify;
mod state;

// spotify desktop client id
pub const CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";

use crate::{
    logic::{
        access_token::AccessToken,
        device_login::DeviceLogin,
        spotify::Spotify,
        state::{SpotifyEvent, SpotifyState},
    },
    state::{AppState, UIEvent, UIState},
};
use anyhow::{Result, anyhow};
use futures_util::{FutureExt, select};
use reqwest::Client;
use std::{fs, sync::Arc, time::Duration};
use tokio::{
    runtime::Handle,
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

pub struct Logic {
    ui_state: UIState,
}

impl Logic {
    pub fn new(ui_state: UIState) -> Self {
        Self { ui_state }
    }

    pub fn spawn(self, runtime: &Handle) -> JoinHandle<Result<()>> {
        runtime.spawn(async move {
            match self.main().await {
                Ok(()) => Ok(()),
                Err(err) => {
                    paris::error!("{err:?}");
                    return Err(err);
                }
            }
        })
    }

    async fn main(mut self) -> Result<()> {
        self.ui_state.set_app_state(AppState::Loading)?;

        let client = Client::new();
        let access_token = match (async || {
            if !fs::exists("spotify.token")? {
                return Err(anyhow!("spotify.token does not exist"));
            }

            let mut t = AccessToken::load(&fs::read("spotify.token")?)?;
            if t.has_expired()
                && let Err(error) = t.refresh(&client).await
            {
                paris::error!("{error}");
                return Err(anyhow!("access token has expired"));
            }

            Ok(t)
        })()
        .await
        {
            Ok(t) => t,
            Err(_) => {
                let mut device_login = DeviceLogin::new();
                self.ui_state.set_login(device_login.device_code().await?)?;
                self.ui_state.set_app_state(AppState::Login)?;

                device_login.wait_token().await?
            }
        };

        fs::write("spotify.token", access_token.store()?)?;
        paris::log!("Signed in with {access_token:?}");
        let (spotify, initial) = Spotify::new(client, access_token).await?;
        let spotify = Arc::new(spotify);
        let mut spotify_state = SpotifyState::new(spotify.clone());
        spotify_state.set_cluster(initial).await?;

        let mut ping_interval = interval(Duration::from_secs(30));
        ping_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        self.ui_state.set_app_state(AppState::Ready)?;

        loop {
            select! {
                event = self.ui_state.wait_event().fuse() => {
                    match event? {
                        UIEvent::ProgressChanged { progress } => spotify.set_progress(progress).await?,
                        UIEvent::PlayingChanged { playing } => spotify.set_playing(playing).await?,
                        UIEvent::ShuffleChanged { shuffle } => spotify.set_shuffle(shuffle).await?,
                        UIEvent::RepeatChanged { repeat } => spotify.set_repeat(repeat).await?,
                        UIEvent::NextTrack => spotify.next_track().await?,
                        UIEvent::PreviousTrack => spotify.prev_track().await?,
                        _ => {}
                    };
                }
                event = spotify_state.wait_event().fuse() => {
                    match event? {
                        SpotifyEvent::ClusterUpdate { image_url, title, artist, year, length, progress, playing } => {
                            self.ui_state.sync_progress(progress)?;
                            self.ui_state.sync_playing(playing)?;

                            self.ui_state
                                .set_track(title, artist, year, length)?;

                            self.ui_state
                                .set_artwork(image_url)
                                .await?;
                        }
                    };
                }
                msg = spotify._next_msg().fuse() => {
                    spotify.process(&msg?, &mut spotify_state).await?;
                }
                _ = ping_interval.tick().fuse() => {
                    spotify.ping().await?;
                }
            };
        }
    }
}
