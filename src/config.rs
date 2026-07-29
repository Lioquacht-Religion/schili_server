// config.rs

use serde::Deserialize;
use tokio::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn get_config() -> &'static Config {
    CONFIG
        .get_or_init(async || {
            toml::from_str::<Config>(
                &std::fs::read_to_string("cfg.toml").expect("Could not open cfg.toml file."),
            )
            .expect("Could not deserialize cfg.toml file.")
        })
        .await
}

#[derive(Deserialize)]
pub struct Config {
    pub http_service: HttpServiceConfig,
    database: DBConfig,
    pub mqtt: MQTTConfig,
    pub email: EmailConfig,
}

#[derive(Deserialize)]
pub struct HttpServiceConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
}

#[derive(Deserialize)]
pub struct DBConfig {
    connection_string: String,
}

#[derive(Deserialize)]
pub struct MQTTConfig {
    pub(crate) broker_id: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) passw: String,
}

#[derive(Deserialize)]
pub struct EmailConfig {
    pub(crate) smtp_server: String,
    pub(crate) smtp_user: String,
    pub(crate) smtp_passw: String,
    pub(crate) author_name: String,
    pub(crate) from_address: String,
    pub(crate) to_addresses: Vec<String>,
}

impl Config {
    pub fn get_db_connect_string(&self) -> &str {
        &self.database.connection_string
    }
}
