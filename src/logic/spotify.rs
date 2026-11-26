use crate::logic::{
    access_token::AccessToken, apresolve::APResolve, device_id::DeviceId, state::SpotifyState,
};
use anyhow::{Context, Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use futures_util::{SinkExt, StreamExt};
use librespot_protocol::connect::{
    Capabilities, Cluster, ClusterUpdate, Device, DeviceInfo, MemberType, PutStateRequest,
};
use protobuf::{Message, MessageField};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::Value;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{self, client::IntoClientRequest},
};

#[derive(serde::Deserialize, Debug)]
struct DealerMessage {
    #[serde(default)]
    headers: HashMap<String, Value>,
    #[serde(default)]
    payloads: Vec<String>,
    // method: String,
    r#type: String,
    #[serde(default)]
    uri: String,
}

pub struct Spotify {
    state: SpotifyState,
    client: Client,
    token: AccessToken,
    apresolve: APResolve,
    device_id: DeviceId,
    connection: Option<String>,
    pub dealer: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Spotify {
    pub async fn new(state: SpotifyState, client: Client, token: AccessToken) -> Result<Self> {
        let apresolve = APResolve::fetch(&client).await?;
        let device_id = DeviceId::new().await?;

        let mut this = Self {
            state,
            client,
            token,
            apresolve,
            device_id,
            connection: None,
            dealer: None,
        };

        this.init_dealer_socket().await?;
        this.init_device().await?;

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

    pub async fn _next_msg(&mut self) -> Result<String> {
        let Some(ref mut stream) = self.dealer else {
            return Err(anyhow!("no available stream"));
        };

        Ok(stream
            .select_next_some()
            .await?
            .into_text()?
            .as_str()
            .to_string())
    }

    /// processes websocket messages
    pub async fn process(&mut self, msg: &str) -> Result<()> {
        let msg = serde_json::from_str::<DealerMessage>(msg)?;

        match msg {
            // DealerMessage { r#type, .. } if r#type == "pong" => paris::info!("pong"),
            DealerMessage { payloads, uri, .. } if uri == "hm://connect-state/v1/cluster" => {
                for payload in payloads {
                    let bytes = BASE64_STANDARD.decode(payload)?;
                    let update = ClusterUpdate::parse_from_bytes(&bytes)?;
                    self.state.update_cluster(update).await?;
                }
            }
            _ => {}
        };

        Ok(())
    }

    /// call every 30 seconds to maintain connection
    pub async fn ping(&mut self) -> Result<()> {
        let Some(ref mut stream) = self.dealer else {
            return Err(anyhow!("no available stream"));
        };

        // paris::info!("ping!");
        stream
            .send(tungstenite::Message::Text("{\"type\":\"ping\"}".into()))
            .await?;

        Ok(())
    }

    async fn init_dealer_socket(&mut self) -> Result<()> {
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

        paris::log!("Connected with id {:?}", self.connection);
        self.dealer = Some(stream);

        Ok(())
    }

    async fn init_device(&mut self) -> Result<()> {
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
                self.device_id
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
        self.state.set_cluster(cluster).await?;

        Ok(())
    }
}
