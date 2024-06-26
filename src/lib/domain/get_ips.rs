use dotenvy::dotenv;
use std::env;

pub async fn load_ips() -> Vec<String> {
    dotenv().ok();
    env::var("LGV_IPS").unwrap_or_default()
        .split(',')
        .map(|s| s.to_string())
        .collect()
}

