use std::net::IpAddr;

use directories::ProjectDirs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;

pub fn get_client() -> Client {
    Client::new()
}

pub async fn get_attached_devices(client: &Client) -> Result<AttachedDevices, Error> {
    const PATH: &str = "/ajax/get_attached_devices";
    let config_path = ProjectDirs::from("com", "rfm", "orbi-helper")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .ok_or_else(|| Error::InvalidConfig("config dir not found".into()))?;
    if !config_path.exists() {
        return Err(Error::InvalidConfig(format!(
            "No file exists at {}",
            config_path.display()
        )));
    }
    let config = fs::read_to_string(&config_path).await?;
    let config: Config = toml::from_str(&config)?;
    let response_body = client
        .post(&format!("http://orbilogin.com{}", PATH))
        .basic_auth(&config.username, Some(&config.password))
        .header("Content-Length", "0")
        .send()
        .await?
        .text()
        .await
        .unwrap();
    // eprintln!("{response_body}");
    let response: AttachedDevices = serde_json::from_str(&response_body)?;
    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachedDevices {
    pub satellites: Vec<Satellite>,
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Satellite {
    mac: String,
    #[serde(rename = "type")]
    kind: String,
    model: String,
    name: String,
    ip: Option<IpAddr>,
    connection_type: String,
    status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub mac: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub model: String,
    pub name: String,
    pub ip: String,
    pub connection_type: String,
    #[serde(alias = "ConnectedOrbi")]
    pub connected_orbi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
