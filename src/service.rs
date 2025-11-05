// service.rs

use anyhow::anyhow;
use schili_api::api;
use sqlx::{Pool, Postgres};

use crate::{api_db_conv::ModelInto, repository};

pub async fn add_sensor(
    pool: &Pool<Postgres>,
    api_sensor: &api::Sensor,
) -> anyhow::Result<repository::Sensor> {
    let mut db_sensor: repository::Sensor = (&*api_sensor).model_into();
    repository::insert_sensor_with_sensor_types(&pool, &mut db_sensor)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be inserted.",
                db_sensor.sensor_reference
            )
        })?;
    Ok(db_sensor)
}

pub async fn insert_temperatures_all(
    pool: &Pool<Postgres>,
    api_temp_measures: &api::SensorTempMeasurements,
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temps): (String, Vec<repository::Temperature>) =
        (&*api_temp_measures).model_into();
    let sensor: repository::Sensor = repository::find_sensor_by_ref(&pool, &sensor_ref)
        .await
        .map_err(|_| anyhow!("Could not find sensor by reference='{}'.", sensor_ref))?;
    repository::insert_sensor_temperature_measures(&pool, sensor.sensor_id, &mut db_temps)
        .await
        .map_err(|_| {
            anyhow!(
                "Could not add temperature measurements for sensor with reference='{}'.",
                sensor_ref
            )
        })?;

    Ok(())
}

pub async fn get_sensor_temperatures_all(
    pool: &Pool<Postgres>,
    sensor_ref: String,
) -> anyhow::Result<api::SensorTempMeasurements> {
    let sensor = repository::find_sensor_by_ref(&pool, &sensor_ref)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be found.",
                &sensor_ref
            )
        })?;
    let temps = repository::find_sensor_temperature_measures(&pool, sensor.sensor_id)
        .await
        .map_err(|_| {
            anyhow!(
                "Error fetching temperature measurements for sensor with reference='{}'.",
                &sensor_ref
            )
        })?;
    let sensor_temps = (sensor_ref.to_owned(), temps);
    let api_temps: api::SensorTempMeasurements = sensor_temps.model_into();
    Ok(api_temps)
}
