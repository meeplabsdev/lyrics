use crate::logic::CLIENT_ID;
use anyhow::{Result, anyhow};
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Client;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum RefreshTokenResponse {
    Success {
        refresh_token: String,
        access_token: String,
        expires_in: u16,
    },
    Error {
        error: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AccessToken {
    pub refresh_token: String,
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub username: String,
    pub expires: DateTime<Utc>,
}

impl AccessToken {
    pub fn has_expired(&self) -> bool {
        Utc::now() + TimeDelta::seconds(60) >= self.expires
    }

    pub async fn refresh(&mut self, client: &Client) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("client_id", CLIENT_ID);
        params.insert("refresh_token", &self.refresh_token);

        let res = client
            .post("https://accounts.spotify.com/api/token")
            .form(&params)
            .send()
            .await?
            .json::<RefreshTokenResponse>()
            .await?;

        match res {
            RefreshTokenResponse::Success {
                refresh_token,
                access_token,
                expires_in,
            } => {
                self.refresh_token = refresh_token;
                self.access_token = access_token;
                self.expires = Utc::now() + TimeDelta::seconds(expires_in as i64);
            }
            RefreshTokenResponse::Error { error } => return Err(anyhow!(error)),
        }

        Ok(())
    }

    pub fn load(data: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(data)?)
    }

    pub fn store(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(self)?)
    }
}
