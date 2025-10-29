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
    database: DBConfig,
}

#[derive(Deserialize)]
pub struct DBConfig {
    connection_string: String,
}

impl Config {
    pub fn get_db_connect_string(&self) -> &str {
        &self.database.connection_string
    }
}
