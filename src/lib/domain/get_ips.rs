use dotenvy::dotenv;
use std::env;
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use derive_more::Display;

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ILGVType {
    pub ip: String,
    pub tc: String,
    pub lid_tag: String,
    pub x_tag: String,
    pub y_tag: String,
    pub status_tag: String,
    pub ams_ip: String,
    pub ams_port: u16,
}

impl ILGVType {
    pub fn new(ip: String, tc: String) -> Self {
        dotenv().ok();
        match tc.as_str() {
            "twincat2" => Self {
                ip: ip.clone(),
                tc,
                lid_tag: env::var("TC2_LID").expect("TC2_LID not set!"),
                x_tag: env::var("TC2_X").expect("TC2_X not set!"),
                y_tag: env::var("TC2_Y").expect("TC2_Y not set!"),
                status_tag: env::var("TC2_STATUS").expect("TC2_STATUS not set!"),
                ams_ip: ip.clone(),
                ams_port: 801,
            },
            "twincat3" => Self {
                ip,
                tc,
                lid_tag: env::var("TC3_LID").expect("TC3_LID not set!"),
                x_tag: env::var("TC3_X").expect("TC3_X not set!"),
                y_tag: env::var("TC3_Y").expect("TC3_Y not set!"),
                status_tag: env::var("TC3_STATUS").expect("TC3_STATUS not set!"),
                ams_ip: env::var("AMS_IP").expect("AMS_IP not set!"),
                ams_port: 851,
            },
            _ => panic!("Unknown TwinCAT type"),
        }
    }
}

#[derive(Deserialize)]
struct LGVConfig {
    ip: String,
    tc: String,
}

pub async fn load_lgv_config() -> Result<Vec<ILGVType>> {
    dotenv().ok();
    let config_path = env::var("LGV_CONFIG_PATH").expect("LGV_CONFIG_PATH must be set in .env file");
    let config_content = fs::read_to_string(config_path)?;
    let configs: Vec<LGVConfig> = serde_json::from_str(&config_content)?;

    Ok(configs.into_iter().map(|config| ILGVType::new(config.ip, config.tc)).collect())
}

pub async fn load_ips() -> Vec<String> {
    dotenv().ok();
    env::var("LGV_IPS").unwrap_or_default()
        .split(',')
        .map(|s| s.to_string())
        .collect()
}