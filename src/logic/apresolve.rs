use anyhow::Result;
use reqwest::Client;
use url::Url;

#[derive(serde::Deserialize)]
pub struct APResolve {
    #[serde(rename = "dealer-g2")]
    dealerg2: Vec<Url>,
    spclient: Vec<Url>,
}

impl APResolve {
    pub async fn fetch(client: &Client) -> Result<Self> {
        let mut this = client
            .get("https://apresolve.spotify.com/?type=dealer-g2&type=spclient")
            .send()
            .await?
            .error_for_status()?
            .json::<Self>()
            .await?;

        for url in this.dealerg2.iter_mut() {
            if url.scheme().contains('.') {
                // gets parsed wrong so we have to fix it i guess
                *url = Url::parse(&format!("wss://{}", url.as_str()))?;
            }
        }

        for url in this.spclient.iter_mut() {
            if url.scheme().contains('.') {
                *url = Url::parse(&format!("https://{}", url.as_str()))?;
            }
        }

        Ok(this)
    }

    // TODO: switch endpoint on failure
    pub fn dealerg2(&self) -> Option<&Url> {
        self.dealerg2.get(0)
    }

    pub fn spclient(&self) -> Option<&Url> {
        self.spclient.get(0)
    }
}
