// service.rs

use anyhow::anyhow;
use bigdecimal::{BigDecimal, Signed};
use chrono::Local;
use log::info;
use schili_api::api::{
    self, GetSensorSimpleMeasuresIntervalsRange, GetSensorSimpleMeasuresRange, SensorType, SensorTypeWarnLimits, SensorWithWarnLimitsAndDBId, SimpleMeasurement,
};
use sqlx::{Pool, Postgres};

use crate::{
    SensorCache, api_db_conv::ModelInto, config, email, repository::{
        self, AvgMeasureTimeInterval, DBSimpleMeasurement, SensorMeasurement
    },
};

// ++++++++++++++ Sensor - SECTION +++++++++++++++++++++

pub async fn add_sensor(
    pool: &Pool<Postgres>,
    api_sensor: &api::SensorWithWarnLimits,
) -> anyhow::Result<repository::Sensor> {
    let (mut db_sensor, warn_limits) = (&*api_sensor).model_into();
    repository::insert_sensor_with_sensor_types(&pool, &mut db_sensor, &warn_limits)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be inserted.",
                db_sensor.sensor_reference
            )
        })?;
    Ok(db_sensor)
}

pub async fn get_sensor(pool: &Pool<Postgres>, sensor_ref: &str) -> anyhow::Result<api::SensorWithTypes> {
    let db_sensor = repository::find_sensor_and_types_by_ref(&pool, sensor_ref)
        .await
        .map_err(|e| {
            anyhow!("Sensor with reference='{sensor_ref}' could not be found. Error: {e}",)
        })?;
    let api_sensor = (&db_sensor).model_into();
    Ok(api_sensor)
}

pub async fn get_all_sensors(pool: &Pool<Postgres>) -> anyhow::Result<Vec<api::SensorWithTypes>> {
    let db_sensors = repository::find_all_sensors(&pool)
        .await
        .map_err(|e| anyhow!("Sensors could not be fetched. Error: {e}",))?;
    let api_sensor = db_sensors.iter().map(|s| s.model_into()).collect();
    Ok(api_sensor)
}

pub async fn get_all_sensors_filtered(
    pool: &Pool<Postgres>,
    sensor_name_part: &str,
    sensor_types: &[api::SensorType],
) -> anyhow::Result<Vec<api::SensorWithTypes>> {
    let sensor_types: Vec<repository::SensorTypeDB> =
        sensor_types.iter().map(|st| (*st).into()).collect();
    let db_sensors =
        repository::find_all_sensors_with_filter(&pool, sensor_name_part, &sensor_types)
            .await
            .map_err(|e| {
                anyhow!(
                    "Sensors with search string='{}' could not be found. Error: {}",
                    sensor_name_part,
                    e
                )
            })?;
    let api_sensors = db_sensors.iter().map(|s| s.model_into()).collect();
    Ok(api_sensors)
}

// ++++++++++++++ Temperature - SECTION +++++++++++++++++++++

pub async fn insert_temperature_w_sensor<'a, 'b>(
    pool: &'a Pool<Postgres>,
    api_temp_measure: &api::SensorSingleSimpleMeasure,
    sensor_cache: SensorCache,
) -> anyhow::Result<()> {
    let sensor_ref: &str = api_temp_measure.sensor_reference.as_str();

    let sensor
        = sensor_cache.get_sensor_by_ref(pool, sensor_ref).await?;
    insert_measurement(
        pool,
        sensor.sensor_id,
        SensorType::Temperature,
        &api_temp_measure.measure,
    )
    .await?;
    Ok(())
}

pub(crate) async fn add_measurement_check_warn_limits(
    pool: &Pool<Postgres>,
    sensor_type: SensorType,
    sensor: &SensorWithWarnLimitsAndDBId,
    warn_limits: Option<&SensorTypeWarnLimits>,
    api_measure: &api::SimpleMeasurement,
) -> anyhow::Result<()> {
    let mut m = (sensor_type, api_measure).model_into();
    let cur_datetime = &Local::now();

    let cur_measurement = &api_measure.measurement;
    check_warn_limits_and_send_emails(
        pool, sensor, warn_limits, cur_measurement, cur_datetime
    ).await?;
    repository::insert_single_sensor_measurement(
        pool, sensor.sensor_id, &mut m)
        .await
}

async fn check_warn_limits_and_send_emails(
    pool: &Pool<Postgres>,
    sensor: &SensorWithWarnLimitsAndDBId,
    warn_limits: Option<&SensorTypeWarnLimits>,
    cur_measurement: &BigDecimal,
    cur_datetime: &chrono::DateTime<Local>,
) -> anyhow::Result<()>{
    if let Some(warn_limits) = warn_limits{
        handle_measurement_warning_email(
            pool, &sensor, warn_limits, cur_measurement, cur_datetime).await?;
    }
    Ok(())
}

pub(crate) async fn insert_measurement<'a, 'b>(
    pool: &'a Pool<Postgres>,
    sensor_id: i32,
    sensor_type: SensorType,
    api_measure: &api::SimpleMeasurement,
) -> anyhow::Result<()> {
    let mut m: SensorMeasurement = (sensor_type, api_measure).model_into();
    match sensor_type {
        SensorType::Co2 => Err(anyhow!("Measurement type CO2 is not supported.")),
        _sensor_type => repository::insert_single_sensor_measurement(pool, sensor_id, &mut m).await

    }
    .map_err(|e| {
        anyhow!(
            "Could not add {} measurement for sensor with id='{}'. Error: {e}",
            sensor_type.to_str(),
            sensor_id
        )
    })?;

    info!(
        "sensor {} measurement: {}",
        sensor_type.to_str(),
        serde_json::to_string(api_measure)?
    );
    Ok(())
}

enum WarnLimitStatus {
    HighMeasure,
    LowMeasure,
    StrongIncreaseMeasure {
        prev_temp: BigDecimal,
        diff_temp: BigDecimal,
    },
    StrongDecreaseMeasure {
        prev_temp: BigDecimal,
        diff_temp: BigDecimal,
    },
    NormalMeasure,
}

async fn handle_measurement_warning_email(
    pool: &Pool<Postgres>,
    sensor: &SensorWithWarnLimitsAndDBId,
    warn_limits: &SensorTypeWarnLimits,
    cur_measurement: &BigDecimal,
    cur_datetime: &chrono::DateTime<Local>,
) -> anyhow::Result<()>{
    let temp_status: WarnLimitStatus = if let Ok(repository::SimpleMeasurement {
        measurement: prev_temp,
        measure_time: prev_time,
        ..
    }) =
        repository::find_sensor_last_measurement_before_at_datetime(pool, sensor.sensor_id, warn_limits.sensor_type.into(), cur_datetime)
            .await
    {
        if let Some(high_limit) = &warn_limits.warn_high 
            && cur_measurement >= &high_limit.limit
            && !(&prev_temp >= &high_limit.limit
                && cur_datetime.naive_local() - prev_time <= high_limit.time_duration)
        {
            WarnLimitStatus::HighMeasure
        } else if let Some(low_limit) = &warn_limits.warn_low 
            && cur_measurement <= &low_limit.limit
            && !(&prev_temp >= &low_limit.limit
                && cur_datetime.naive_local() - prev_time <= low_limit.time_duration)
        {
            WarnLimitStatus::LowMeasure
        } else {
            let diff_temp: BigDecimal = cur_measurement - &prev_temp;
            if let Some(warn_increase) = &warn_limits.warn_increase 
                && &diff_temp >= &warn_increase.limit {
                WarnLimitStatus::StrongIncreaseMeasure {
                    prev_temp,
                    diff_temp,
                }
            } else if let Some(warn_decrease) = &warn_limits.warn_decrease
                && diff_temp.is_negative() 
                && &diff_temp.abs() >= &warn_decrease.limit {
                WarnLimitStatus::StrongDecreaseMeasure {
                    prev_temp,
                    diff_temp,
                }
            } else {
                WarnLimitStatus::NormalMeasure
            }
        }
    } else {
        if let Some(warn_high) = &warn_limits.warn_high 
            && cur_measurement >= &warn_high.limit {
            WarnLimitStatus::HighMeasure
        } else if let Some(warn_low) = &warn_limits.warn_low 
            && cur_measurement <= &warn_low.limit {
            WarnLimitStatus::LowMeasure
        } else {
            WarnLimitStatus::NormalMeasure
        }
    };

    let email_config = &config::get_config().await.email;
    match temp_status {
        WarnLimitStatus::HighMeasure => email::send_high_measurement_warning_email(&email_config, warn_limits.sensor_type, &cur_measurement),
        WarnLimitStatus::LowMeasure => email::send_low_measurement_warning_email(&email_config, warn_limits.sensor_type, &cur_measurement),
        WarnLimitStatus::StrongIncreaseMeasure {
            prev_temp,
            diff_temp,
        } => {
            email::send_measured_strong_increase_email(&email_config, warn_limits.sensor_type, &cur_measurement, &prev_temp, &diff_temp)
        }
        WarnLimitStatus::StrongDecreaseMeasure {
            prev_temp,
            diff_temp,
        } => {
            email::send_measured_strong_decrease_email(&email_config, warn_limits.sensor_type, &cur_measurement, &prev_temp, &diff_temp)
        }
        WarnLimitStatus::NormalMeasure => {}
    }

    Ok(())
}

pub async fn insert_simple_measurement<'a, 'b, 'c, T, Fut, F>(
    pool: &'a Pool<Postgres>,
    measure_name: &str,
    sensor_ref: &str,
    db_temp: &'b mut T,
    db_insert: F,
) -> anyhow::Result<()>
where
    T: DBSimpleMeasurement + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'c,
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

pub async fn get_sensor_measurements_in_range(
    pool: &Pool<Postgres>,
    sensor_temp_range: &GetSensorSimpleMeasuresRange,
    measurement_kind: SensorType,
) -> anyhow::Result<api::SensorSimpleMeasurements> {
    let sensor = repository::find_sensor_by_ref(&pool, &sensor_temp_range.sensor_reference)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be found.",
                &sensor_temp_range.sensor_reference,
            )
        })?;
    let result = match measurement_kind {

        SensorType::Co2 => todo!(),
        _sensor_type=> {
            repository::find_sensor_measurements_in_timerange(
                &pool,
                sensor.sensor_id,
                measurement_kind.into(),
                &sensor_temp_range.start_datetime,
                &sensor_temp_range.end_datetime,
            )
            .await
        }
    };
    let temps = result.map_err(|e| {
        anyhow!(
            "Error fetching temperature measurements for sensor with reference='{}'. Error: {e}",
            &sensor_temp_range.sensor_reference
        )
    })?;
    let api_temps = api::SensorSimpleMeasurements {
        sensor_reference: sensor.sensor_reference,
        measurements: temps
            .into_iter()
            .map(
                |repository::SimpleMeasurement {
                     measurement,
                     measure_time,
                 }| SimpleMeasurement {
                    measure_time: measure_time.and_utc(),
                    measurement,
                },
            )
            .collect(),
    };
    Ok(api_temps)
}

pub async fn get_sensor_avg_measurements_by_intervals_in_range(
    pool: &Pool<Postgres>,
    sensor_temp_range: &GetSensorSimpleMeasuresIntervalsRange,
    measurement_kind: SensorType,
) -> anyhow::Result<api::SensorSimpleMeasurements> {
    let sensor = repository::find_sensor_by_ref(&pool, &sensor_temp_range.sensor_reference)
        .await
        .map_err(|_| {
            anyhow!(
                "Sensor with reference='{}' could not be found.",
                &sensor_temp_range.sensor_reference,
            )
        })?;
    let result = match measurement_kind {
        SensorType::Co2 => todo!(),
        _sensor_type => {
            repository::find_sensor_avg_measurements_by_intervals_in_timerange(
                &pool,
                sensor.sensor_id,
                measurement_kind.into(),
                &sensor_temp_range.start_datetime,
                &sensor_temp_range.end_datetime,
                sensor_temp_range.interval,
            )
            .await
        }
    };
    let temps = result.map_err(|e| {
        anyhow!(
            "Error fetching temperature measurements for sensor with reference='{}'. Error: {e}",
            &sensor_temp_range.sensor_reference
        )
    })?;
    let api_temps = api::SensorSimpleMeasurements {
        sensor_reference: sensor.sensor_reference,
        measurements: temps
            .into_iter()
            .map(
                |AvgMeasureTimeInterval {
                     timestamp_from,
                     avg_measurement,
                 }| SimpleMeasurement {
                    measure_time: timestamp_from.and_utc(),
                    measurement: avg_measurement,
                },
            )
            .collect(),
    };
    Ok(api_temps)
}

// ++++++++++++++ CO2 - SECTION +++++++++++++++++++++

pub async fn insert_co2(
    pool: &Pool<Postgres>,
    sensor_cache: SensorCache,
    api_co2_measure: &api::SensorSingleCo2Measure,
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_co2): (String, repository::Co2) = (&*api_co2_measure).model_into();
    let sensor = sensor_cache.get_sensor_by_ref(pool, &sensor_ref).await?;
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

// ++++++++++++++ BUNDLED MEASUREMENTS - SECTION +++++++++++++++++++++

pub async fn insert_bundled_measurements(
    pool: &Pool<Postgres>,
    sensor_cache: SensorCache,
    api_measurements: &mut api::SensorTypedSimpleMeasurements,
) -> anyhow::Result<(), Vec<anyhow::Error>> {
    let sensor_ref = &api_measurements.sensor_reference;
    let sensor = sensor_cache.get_sensor_by_ref(pool, sensor_ref).await.map_err(|e| vec![e])?;

    let mut errors: Vec<anyhow::Error> = Vec::new();
    for m in api_measurements.measurements.iter() {
        let warn_limits = sensor.sensor_w_warn_limits.sensor_type_warn_limits.get(&m.sensor_type);
        if let Err(e) = add_measurement_check_warn_limits(
            pool, m.sensor_type, &sensor, warn_limits, &m.measure).await
        {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ++++++++++++++ ERROR - SECTION +++++++++++++++++++++

pub async fn insert_sensor_error(
    pool: &Pool<Postgres>,
    sensor_cache: SensorCache,
    api_sensor_error: &api::SensorError,
) -> anyhow::Result<()> {
    let (sensor_ref, mut db_sensor_error): (String, repository::SensorError) =
        (&*api_sensor_error).model_into();
    let sensor = sensor_cache.get_sensor_by_ref(pool, &sensor_ref).await?;
    repository::insert_single_sensor_error(&pool, sensor.sensor_id, &mut db_sensor_error)
        .await
        .map_err(|_| {
            anyhow!(
                "Could not add sensor error for sensor with reference='{}'.",
                sensor_ref
            )
        })?;

    Ok(())
}
