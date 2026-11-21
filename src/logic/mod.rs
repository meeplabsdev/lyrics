mod access_token;
mod apresolve;
mod device_login;
mod spotify;

// spotify desktop client id
pub const CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";

use crate::{
    logic::{access_token::AccessToken, device_login::DeviceLogin, spotify::Spotify},
    state::{AppState, Event, State},
};
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::fs;
use tokio::{runtime::Handle, task::JoinHandle};

pub struct Logic {
    state: State,
}

impl Logic {
    pub fn new(state: State) -> Self {
        Self { state }
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
        self.state.set_app_state(AppState::Loading)?;

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
                self.state.set_login(device_login.device_code().await?)?;
                self.state.set_app_state(AppState::Login)?;
                device_login.wait_token().await?
            }
        };

        fs::write("spotify.token", access_token.store()?)?;
        paris::log!("Signed in with {access_token:?}");
        let spotify = Spotify::new(client, access_token).await?;

        self.state.set_app_state(AppState::Ready)?;

        self.state
            .set_track("Honestly", "THØRNS, Kasane Teto", 2024, 218)?;

        self.state
            .set_artwork("https://i.scdn.co/image/ab67616d0000b27340a96de45049b6714636ef2d")
            .await?;

        self.state.sync_progress(0.4)?;
        self.state.sync_playing(true)?;

        loop {
            match self.state.wait_event().await? {
                Event::NextTrack => spotify.next_track().await?,
                Event::PreviousTrack => spotify.prev_track().await?,
                _ => {}
            };
        }
    }
}
