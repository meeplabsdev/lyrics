use crate::logic::{access_token::AccessToken, apresolve::APResolve};
use anyhow::{Context, Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use librespot_protocol::connect::{
    Capabilities, Cluster, Device, DeviceInfo, MemberType, PutStateRequest,
};
use protobuf::{Message, MessageField};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::{collections::HashMap, fs};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{self, client::IntoClientRequest},
};

#[derive(serde::Deserialize)]
struct DealerMessage {
    headers: HashMap<String, Value>,
    // method: String,
    // r#type: String,
    // uri: String,
}

pub struct Spotify {
    client: Client,
    token: AccessToken,
    apresolve: APResolve,
    device: [u8; 20],
    connection: Option<String>,
    dealer: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    cluster: Option<Cluster>,
}

impl Spotify {
    pub async fn new(client: Client, token: AccessToken) -> Result<Self> {
        let apresolve = APResolve::fetch(&client).await?;
        let device = match (async || {
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
        };

        let mut this = Self {
            client,
            token,
            apresolve,
            device,
            connection: None,
            dealer: None,
            cluster: None,
        };

        this.create_dealer_socket().await?;
        this.create_device().await?;

        Ok(this)
    }

    pub async fn next_track(&self) -> Result<()> {
        paris::warn!("TODO: next_track");

        Ok(())
    }

    pub async fn prev_track(&self) -> Result<()> {
        paris::warn!("TODO: prev_track");

        Ok(())
    }

    async fn create_dealer_socket(&mut self) -> Result<()> {
        let mut url = self
            .apresolve
            .dealerg2()
            .context("no available dealerg2")?
            .to_owned();

        url.query_pairs_mut()
            .append_pair("access_token", &self.token.access_token.clone());

        let req = url.as_str().into_client_request()?;
        let (mut stream, _) = connect_async(req).await?;
        let msg = serde_json::from_str::<DealerMessage>(
            stream.select_next_some().await?.into_text()?.as_str(),
        )?;

        self.connection = Some(
            msg.headers
                .get("Spotify-Connection-Id")
                .context("message did not have Spotify-Connection-Id")?
                .as_str()
                .context("Spotify-Connection-Id was found but not a string")?
                .to_string(),
        );

        // TODO: ping every 30 seconds
        paris::log!("Connected with id {:?}", self.connection);
        stream
            .send(tungstenite::Message::Text("{\"type\":\"ping\"}".into()))
            .await?;

        self.dealer = Some(stream);

        Ok(())
    }

    async fn create_device(&mut self) -> Result<()> {
        let Some(ref connection_id) = self.connection else {
            return Err(anyhow!("create_device before connection established"));
        };

        let mut caps = Capabilities::new();
        caps.can_be_player = false;
        caps.hidden = true;
        caps.needs_full_player_state = true;

        let mut info = DeviceInfo::new();
        info.capabilities = MessageField::some(caps);

        let mut device = Device::new();
        device.device_info = MessageField::some(info);

        let mut req = PutStateRequest::new();
        req.device = MessageField::some(device);
        req.member_type = MemberType::CONNECT_STATE.into();

        let res = self
            .client
            .put(format!(
                "{}connect-state/v1/devices/{}",
                self.apresolve
                    .spclient()
                    .context("no available spclient")?
                    .as_str(),
                hex::encode(self.device)
            ))
            .header(
                AUTHORIZATION,
                format!("{} {}", self.token.token_type, self.token.access_token),
            )
            .header("X-Spotify-Connection-Id", connection_id)
            .header(CONTENT_TYPE, "application/protobuf")
            .body(req.write_to_bytes()?)
            .send()
            .await?
            .error_for_status()?;

        let cluster = Cluster::parse_from_bytes(&res.bytes().await?)?;
        self.cluster = Some(cluster);
        paris::log!("Got cluster {:#?}", self.cluster);

        Ok(())
    }
}
