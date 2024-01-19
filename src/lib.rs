use std::collections::HashMap;

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
    let mut response: AttachedDevices = serde_json::from_str(&response_body)?;
    for device in response.devices.iter_mut() {
        if let Some(mac_override) = config.device_name_overrides.get(&device.mac) {
            device.name = mac_override.to_string();
            continue;
        }
        if let Some(name_override) = config.device_name_overrides.get(&device.name) {
            device.name = name_override.to_string();
            continue;
        }
    }
    Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachedDevices {
    pub satellites: Vec<Device>,
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
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
    #[serde(alias = "ConnectedOrbiMAC")]
    pub connected_orbi_mac: String,
    pub connection_img: String,
    pub backhaul_status_style: String,
    pub backhaul_status: String,
    pub category: String,
    pub status: u32,
    #[serde(alias = "sat_type")]
    pub sat_type: u32,
    #[serde(alias = "led_status")]
    pub led_status: u32,
    #[serde(alias = "led_brightness")]
    pub led_brightness: u32,
    #[serde(alias = "led_sync")]
    pub led_sync: u32,
    #[serde(alias = "voice_reg")]
    pub voice_reg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub device_name_overrides: HashMap<String, String>,
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
