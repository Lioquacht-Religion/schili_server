// service.rs

use anyhow::anyhow;
use log::{error, info};
use schili_api::api::{self, GetSensorSimpleMeasuresRange};
use sqlx::{Pool, Postgres};

use crate::{api_db_conv::ModelInto, repository::{self, AirPressure, BatteryVoltage, ChipTemperature, DBSimpleMeasurement, Humidity, Temperature}};

// ++++++++++++++ Sensor - SECTION +++++++++++++++++++++

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

// ++++++++++++++ Temperature - SECTION +++++++++++++++++++++

pub async fn insert_temperature<'a, 'b>(
    pool: &'a Pool<Postgres>,
    api_temp_measure: &api::SensorSingleSimpleMeasure
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temp): (String, Temperature) =
        (&*api_temp_measure).model_into();

        if let Err(e) = insert_simple_measurement(
            pool, 
            "temperature", 
            &sensor_ref,
            &mut db_temp, 
                    repository::insert_single_sensor_temperature
            )
            .await {
                error!("Could not insert temperatures from mq publish. error: {}", e);
        }

        info!(
            "sensor temps: {}",
            serde_json::to_string(api_temp_measure)?
        );
        Ok(())
}

pub async fn insert_simple_measurement<'a, 'b, 'c, T, Fut, F>(
    pool: &'a Pool<Postgres>,
    measure_name: &str,
    sensor_ref: &str,
    db_temp: &'b mut T,
    db_insert: F) -> anyhow::Result<()> 
where 
    T: DBSimpleMeasurement + 'static,
    Fut: Future<Output = std::result::Result<(), Box<dyn std::error::Error>>> 
    + Send + 'c,
    F: FnOnce(&'a Pool<Postgres>, i32, &'b mut T) -> Fut + 'static,
{
    let sensor: repository::Sensor = repository::find_sensor_by_ref(&pool, &sensor_ref)
        .await
        .map_err(|_| anyhow!("Could not find sensor by reference='{}'.", sensor_ref))?;
    db_insert(&pool, sensor.sensor_id, db_temp)
        .await
        .map_err(|_| {
            anyhow!(
                "Could not add {} measurement for sensor with reference='{}'.",
                measure_name,
                sensor_ref
            )
        })?;
    Ok(())
}

pub async fn insert_temperatures_all(
    pool: &Pool<Postgres>,
    api_temp_measures: &api::SensorSimpleMeasurements,
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

#[deprecated]
pub async fn get_sensor_temperatures_all(
    pool: &Pool<Postgres>,
    sensor_ref: String,
) -> anyhow::Result<api::SensorSimpleMeasurements> {
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
    let api_temps: api::SensorSimpleMeasurements = sensor_temps.model_into();
    Ok(api_temps)
}

pub async fn get_sensor_temperatures_in_range(
    pool: &Pool<Postgres>,
    sensor_temp_range: &GetSensorSimpleMeasuresRange,
) -> anyhow::Result<api::SensorSimpleMeasurements> {
    let sensor = repository::find_sensor_by_ref(
        &pool, &sensor_temp_range.sensor_reference)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be found.",
                &sensor_temp_range.sensor_reference,
            )
        })?;
    let temps = 
        repository::find_sensor_temperature_measures_by_timerange(
            &pool, 
            sensor.sensor_id, 
            &sensor_temp_range.start_datetime,
            &sensor_temp_range.end_datetime
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Error fetching temperature measurements for sensor with reference='{}'.",
                &sensor_temp_range.sensor_reference
            )
        })?;
    let sensor_temps = (
        sensor_temp_range.sensor_reference.to_owned(), 
        temps);
    let api_temps: api::SensorSimpleMeasurements = sensor_temps.model_into();
    Ok(api_temps)
}

// ++++++++++++++ Humidity - SECTION +++++++++++++++++++++

pub async fn insert_humidity<'a, 'b>(
    pool: &'a Pool<Postgres>,
    api_temp_measure: &api::SensorSingleSimpleMeasure
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temp): (String, Humidity) =
        (&*api_temp_measure).model_into();

        if let Err(e) = insert_simple_measurement(
            pool, 
            "humidity", 
            &sensor_ref,
            &mut db_temp, 
                    repository::insert_single_sensor_humidity
            )
            .await {
                error!("Could not insert humidities from mq publish. error: {}", e);
        }

        info!(
            "sensor temps: {}",
            serde_json::to_string(api_temp_measure).unwrap()
        );
        Ok(())
}

pub async fn insert_humidities_all(
    pool: &Pool<Postgres>,
    api_temp_measures: &api::SensorSimpleMeasurements,
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temps): (String, Vec<repository::Humidity>) =
        (&*api_temp_measures).model_into();
    let sensor: repository::Sensor = repository::find_sensor_by_ref(&pool, &sensor_ref)
        .await
        .map_err(|_| anyhow!("Could not find sensor by reference='{}'.", sensor_ref))?;
    repository::insert_sensor_humidity_measures(&pool, sensor.sensor_id, &mut db_temps)
        .await
        .map_err(|_| {
            anyhow!(
                "Could not add humidity measurements for sensor with reference='{}'.",
                sensor_ref
            )
        })?;

    Ok(())
}

pub async fn get_sensor_humidities_in_range(
    pool: &Pool<Postgres>,
    sensor_temp_range: &GetSensorSimpleMeasuresRange,
) -> anyhow::Result<api::SensorSimpleMeasurements> {
    let sensor = repository::find_sensor_by_ref(
        &pool, &sensor_temp_range.sensor_reference)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be found.",
                &sensor_temp_range.sensor_reference,
            )
        })?;
    let temps = 
        repository::find_sensor_humidity_measures_by_timerange(
            &pool, 
            sensor.sensor_id, 
            &sensor_temp_range.start_datetime,
            &sensor_temp_range.end_datetime
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Error fetching humidity measurements for sensor with reference='{}'.",
                &sensor_temp_range.sensor_reference
            )
        })?;
    let sensor_temps = (
        sensor_temp_range.sensor_reference.to_owned(), 
        temps);
    let api_temps: api::SensorSimpleMeasurements = sensor_temps.model_into();
    Ok(api_temps)
}

// ++++++++++++++ Airpressure - SECTION +++++++++++++++++++++

pub async fn insert_airpressure<'a>(
    pool: &'a Pool<Postgres>,
    api_temp_measure: &api::SensorSingleSimpleMeasure
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temp): (String, AirPressure) =
        (&*api_temp_measure).model_into();

        if let Err(e) = insert_simple_measurement(
            pool, 
            "air pressure", 
            &sensor_ref,
            &mut db_temp, 
                    repository::insert_single_sensor_airpressure
            )
            .await {
                error!("Could not insert air pressures from mq publish. error: {}", e);
        }

        info!(
            "sensor air pressures: {}",
            serde_json::to_string(api_temp_measure).unwrap()
        );
        Ok(())
}

pub async fn insert_airpressure_all(
    pool: &Pool<Postgres>,
    api_temp_measures: &api::SensorSimpleMeasurements,
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temps): (String, Vec<repository::AirPressure>) =
        (&*api_temp_measures).model_into();
    let sensor: repository::Sensor = repository::find_sensor_by_ref(&pool, &sensor_ref)
        .await
        .map_err(|_| anyhow!("Could not find sensor by reference='{}'.", sensor_ref))?;
    repository::insert_sensor_airpressure_measures(&pool, sensor.sensor_id, &mut db_temps)
        .await
        .map_err(|_| {
            anyhow!(
                "Could not add air pressure measurements for sensor with reference='{}'.",
                sensor_ref
            )
        })?;

    Ok(())
}

pub async fn get_sensor_airpressure_in_range(
    pool: &Pool<Postgres>,
    sensor_temp_range: &GetSensorSimpleMeasuresRange,
) -> anyhow::Result<api::SensorSimpleMeasurements> {
    let sensor = repository::find_sensor_by_ref(
        &pool, &sensor_temp_range.sensor_reference)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be found.",
                &sensor_temp_range.sensor_reference,
            )
        })?;
    let temps = 
        repository::find_sensor_airpressure_measures_by_timerange(
            &pool, 
            sensor.sensor_id, 
            &sensor_temp_range.start_datetime,
            &sensor_temp_range.end_datetime
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Error fetching air pressure measurements for sensor with reference='{}'.",
                &sensor_temp_range.sensor_reference
            )
        })?;
    let sensor_temps = (
        sensor_temp_range.sensor_reference.to_owned(), 
        temps);
    let api_temps: api::SensorSimpleMeasurements = sensor_temps.model_into();
    Ok(api_temps)
}

// ++++++++++++++ Chip temperature - SECTION +++++++++++++++++++++

pub async fn insert_chip_temperature<'a>(
    pool: &'a Pool<Postgres>,
    api_temp_measure: &api::SensorSingleSimpleMeasure
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temp): (String, ChipTemperature) =
        (&*api_temp_measure).model_into();

        if let Err(e) = insert_simple_measurement(
            pool, 
            "chip temperature", 
            &sensor_ref,
            &mut db_temp, 
                    repository::insert_single_sensor_chip_temperature 
        )
            .await {
                error!("Could not insert chip temperature from mq publish. error: {}", e);
        }

        info!(
            "sensor chip temperature: {}",
            serde_json::to_string(api_temp_measure).unwrap()
        );
        Ok(())
}

// ++++++++++++++ Battery voltage - SECTION +++++++++++++++++++++

pub async fn insert_battery_voltage<'a>(
    pool: &'a Pool<Postgres>,
    api_temp_measure: &api::SensorSingleSimpleMeasure
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_temp): (String, BatteryVoltage) =
        (&*api_temp_measure).model_into();

        if let Err(e) = insert_simple_measurement(
            pool, 
            "battery voltage", 
            &sensor_ref,
            &mut db_temp, 
                    repository::insert_single_sensor_battery_voltage
        )
            .await {
                error!("Could not insert battery voltage from mq publish. error: {}", e);
        }

        info!(
            "sensor battery voltage: {}",
            serde_json::to_string(api_temp_measure).unwrap()
        );
        Ok(())
}

// ++++++++++++++ CO2 - SECTION +++++++++++++++++++++

pub async fn insert_co2(
    pool: &Pool<Postgres>,
    api_co2_measure: &api::SensorSingleCo2Measure,
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_co2): (String, repository::Co2) =
        (&*api_co2_measure).model_into();
    let sensor: repository::Sensor = repository::find_sensor_by_ref(&pool, &sensor_ref)
        .await
        .map_err(|_| anyhow!("Could not find sensor by reference='{}'.", sensor_ref))?;
    repository::insert_single_sensor_co2_measure(&pool, sensor.sensor_id, &mut db_co2)
        .await
        .map_err(|_| {
            anyhow!(
                "Could not add co2 measurement for sensor with reference='{}'.",
                sensor_ref
            )
        })?;

    Ok(())
}

