use anyhow::{Context, Result, anyhow};
use sha1::{Digest, Sha1};
use std::{fmt::Display, fs};

pub struct DeviceId([u8; 20]);

impl DeviceId {
    pub async fn new() -> Result<Self> {
        Ok(Self(
            match (async || {
                if !fs::exists("spotify.device")? {
                    return Err(anyhow!("spotify.device does not exist"));
                }

                let i = &fs::read("spotify.device")?;
                if i.len() != 20 {
                    return Err(anyhow!("spotify.device was not 20 bytes"));
                }

                Ok(*i.as_array().context("failed to parse spotify.device")?)
            })()
            .await
            {
                Ok(i) => i,
                Err(_) => {
                    let i = *Sha1::digest(rand::random::<u128>().to_be_bytes())
                        .as_array()
                        .context("failed to generate device id")?;

                    fs::write("spotify.device", &i)?;

                    i
                }
            },
        ))
    }
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
