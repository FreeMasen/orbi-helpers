use std::{collections::HashMap, path::PathBuf};

use directories::ProjectDirs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;

pub fn get_client() -> Client {
    Client::new()
}

pub fn find_config_path() -> Result<PathBuf, Error> {
    let config_path = ProjectDirs::from("com", "rfm", "orbi-helper")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .ok_or_else(|| Error::InvalidConfig("config dir not found".into()))?;
    if !config_path.exists() {
        return Err(Error::MissingConfigFile(config_path));
    }
    Ok(config_path)
}

pub async fn read_config() -> Result<Config, Error> {
    let config_path = find_config_path()?;
    let config = fs::read_to_string(&config_path).await?;
    Ok(toml::from_str(&config)?)
}

pub async fn save_config(config: &Config) -> Result<(), Error> {
    let path = match find_config_path() {
        Ok(path) => path,
        Err(Error::MissingConfigFile(path)) => {
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
            path
        },
        Err(e) => return Err(e),
    };
    let output = toml::to_string_pretty(&config)?;
    tokio::fs::write(path, output).await?;
    Ok(())
}

pub async fn set_config_name_override(key: String, value: Option<String>) -> Result<(), Error> {
    let mut config = read_config().await.unwrap_or_default();
    if let Some(new_name) = value {
        config.device_name_overrides.entry(key).and_modify(|v| {
            *v = new_name.clone();
        }).or_insert(new_name);
    } else {
        config.device_name_overrides.remove(&key);
    }
    save_config(&config).await
}

pub async fn set_config_username(value: &str) -> Result<(), Error> {
    let mut config = read_config().await.unwrap_or_default();
    config.username = value.to_string();
    save_config(&config).await
}

pub async fn set_config_password(value: &str) -> Result<(), Error> {
    let mut config = read_config().await.unwrap_or_default();
    config.password = value.to_string();
    save_config(&config).await
}

pub async fn get_attached_devices(client: &Client) -> Result<AttachedDevices, Error> {
    const PATH: &str = "/ajax/get_attached_devices";
    let config = read_config().await?;
    let response_body = client
        .post(&format!("http://orbilogin.com{}", PATH))
        .basic_auth(&config.username, Some(&config.password))
        .header("Content-Length", "0")
        .send()
        .await?
        .text()
        .await
        .unwrap();
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub device_name_overrides: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error("Missing config file expected to be at {0:?}")]
    MissingConfigFile(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
