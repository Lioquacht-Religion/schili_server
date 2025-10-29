// database.rs

use std::borrow::Cow;

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::config::{Config, get_config};

pub async fn create_db_pool() -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&get_database_connection_string(get_config().await))
        .await
        .expect("Couldn't create posgresql DB connections pool.")
}

pub fn get_database_connection_string<'a>(config: &'a Config) -> Cow<'a, str> {
    match dotenvy::var("DATABASE_URL") {
        Ok(db_str) if !db_str.is_empty() => Cow::Owned(db_str),
        _ => Cow::Borrowed::<'a>(config.get_db_connect_string()),
    }
}
