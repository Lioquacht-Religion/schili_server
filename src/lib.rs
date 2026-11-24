// lib.rs

use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use moka::future::Cache;
use schili_api::api::{self, SensorWithWarnLimitsAndDBId};
use sqlx::PgPool;

pub mod config;
pub mod http_server;
pub mod mqtt_handler;
pub mod topic_router;

pub mod api_db_conv;
pub mod database;
pub mod email;
pub mod error;
pub mod repository;
pub mod service;

#[derive(Debug, Clone)]
pub struct SensorCache{
    cache: Cache<String, Arc<api::SensorWithWarnLimitsAndDBId>>
}

impl SensorCache{
    pub fn new(max_capacity: u64) -> Self{
        SensorCache{ cache: Cache::new(max_capacity)}
    }

    pub async fn get_sensor_by_ref(&self, pool: &PgPool, sensor_ref: &str)
     -> anyhow::Result<Arc<SensorWithWarnLimitsAndDBId>> {
        let entry = self.cache.entry(sensor_ref.to_owned());
        let sensor
            = entry
            .or_try_insert_with(async{
                repository::find_sensor_with_warnlimits(&pool, &sensor_ref)
                .await
                .map(|(s, w)| 
                    Arc::new(
                        SensorWithWarnLimitsAndDBId::new(
                                  s.sensor_id,
                            &s.sensor_reference, &s.sensor_name,
                            HashMap::from_iter(
                                w.into_iter()
                                .map(|w| (w.sensor_type, w))
                            )
                        )
                    ))
            }
            ).await
            .map_err(|_| anyhow!("Could not find sensor by reference='{}'.", sensor_ref))?;
         Ok(Arc::clone(sensor.value()))
    }

    pub async fn evict_sensor_by_ref(&self, sensor_ref: &str){
        self.cache.remove(sensor_ref).await;
    }
}
