use crate::{
    logic::{CLIENT_ID, access_token::AccessToken},
    utils::ExpiringOption,
};
use anyhow::{Result, anyhow};
use chrono::{TimeDelta, Utc};
use reqwest::Client;
use std::{collections::HashMap, time::Duration};

#[derive(serde::Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    // verification_uri: String,
    // verification_uri_complete: String,
    expires_in: u16,
    interval: u16,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum AccessTokenResponse {
    Success {
        refresh_token: String,
        access_token: String,
        token_type: String,
        scope: String,
        username: String,
        expires_in: u16,
    },
    Error {
        error: String,
    },
}

pub struct DeviceLogin {
    client: Client,
    scope: Vec<&'static str>,
    device_code: ExpiringOption<String>,
    poll_interval: Duration,
}

impl DeviceLogin {
    pub fn new() -> Self {
        let client = Client::new();
        let scope = vec![
            "app-remote-control",
            "playlist-modify",
            "playlist-modify-private",
            "playlist-modify-public",
            "playlist-read",
            "playlist-read-collaborative",
            "playlist-read-private",
            "streaming",
            "transfer-auth-session",
            "sts-content-management",
            "ugc-image-upload",
            "user-follow-modify",
            "user-follow-read",
            "user-library-modify",
            "user-library-read",
            "user-modify",
            "user-modify-playback-state",
            "user-modify-private",
            "user-personalized",
            "user-read-birthdate",
            "user-read-currently-playing",
            "user-read-email",
            "user-read-play-history",
            "user-read-playback-position",
            "user-read-playback-state",
            "user-read-private",
            "user-read-recently-played",
            "user-top-read",
        ];

        Self {
            client,
            scope,
            device_code: ExpiringOption::None,
            poll_interval: Duration::from_secs(5),
        }
    }

    pub async fn device_code(&mut self) -> Result<String> {
        let scope_str = self.scope.join(",");
        let mut params = HashMap::new();
        params.insert("client_id", CLIENT_ID);
        params.insert("scope", &scope_str);

        let res = self
            .client
            .post("https://accounts.spotify.com/oauth2/device/authorize")
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json::<DeviceCodeResponse>()
            .await?;

        self.poll_interval = Duration::from_secs(res.interval as u64);
        self.device_code = ExpiringOption::some_with_life(
            res.device_code,
            Duration::from_secs(res.expires_in as u64),
        );

        Ok(res.user_code)
    }

    pub async fn wait_token(&mut self) -> Result<AccessToken> {
        loop {
            if let Some(token) = self.poll_code().await? {
                return Ok(token);
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn poll_code(&mut self) -> Result<Option<AccessToken>> {
        let Some(device_code) = self.device_code.get() else {
            return Err(anyhow!("expired or no device_code"));
        };

        let mut params = HashMap::new();
        params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
        params.insert("client_id", CLIENT_ID);
        params.insert("device_code", device_code);

        let res = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .form(&params)
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;

        match res {
            AccessTokenResponse::Error { error } => {
                if error == "authorization_pending" {
                    return Ok(None);
                }

                if error == "slow_down" {
                    self.poll_interval += Duration::from_secs(5);
                    return Ok(None);
                }

                Err(anyhow!(error))
            }
            AccessTokenResponse::Success {
                refresh_token,
                access_token,
                token_type,
                scope,
                username,
                expires_in,
            } => Ok(Some(AccessToken {
                refresh_token,
                access_token,
                token_type,
                scope,
                username,
                expires: Utc::now() + TimeDelta::seconds(expires_in as i64),
            })),
        }
    }
}
