use directories::ProjectDirs;
use std::{fs, io, path::PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub ip_version: IpVersion,
    pub listen_ip: String,
    pub listen_port: u16,
    pub bootstrap_ip: Option<String>,
    pub bootstrap_port: Option<u16>,
    pub bootstrap_peer_id: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpVersion {
    Ipv4,
    Ipv6,
}


pub fn config_path() -> PathBuf {
    let proj = ProjectDirs::from("dev", "thenmuste", "p2pchat")
        .expect("Cannot determine config directory");

    proj.config_dir().join("config.json")
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.network.listen_port == 0 {
            return Err("Listen port cannot be 0".into());
        }

        let b = &self.network;
        let bootstrap_complete =
            b.bootstrap_ip.is_some()
                && b.bootstrap_port.is_some()
                && b.bootstrap_peer_id.is_some();

        let bootstrap_empty =
            b.bootstrap_ip.is_none()
                && b.bootstrap_port.is_none()
                && b.bootstrap_peer_id.is_none();

        if !(bootstrap_complete || bootstrap_empty) {
            return Err(
                "Bootstrap config must have ip, port, and peer_id together"
                    .into(),
            );
        }

        Ok(())
    }

    pub fn load() -> io::Result<Self> {
        let path = config_path();

        if !path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let data = fs::read_to_string(path)?;
        let cfg: Self = serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(cfg)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(path, json)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
             network: NetworkConfig {
                ip_version: IpVersion::Ipv4,
                listen_ip: "0.0.0.0".into(),
                listen_port: 8000,
                bootstrap_ip: Some("127.0.0.1".to_string()),
                bootstrap_port: Some(8000),
                bootstrap_peer_id: Some("12D3KooWJ5VBBryqyPrBXAd28fk9KsH3pXdiXshH6gpsLWWi6WiH".to_string()),
            },
        }
    }
}

