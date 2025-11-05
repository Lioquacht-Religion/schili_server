// repository.rs

use std::collections::HashSet;

use chrono::NaiveDateTime;
use sqlx::{PgPool, Pool, Postgres, types::BigDecimal};

pub async fn start_sql_query(
    pool: &Pool<Postgres>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(pool)
        .await?;

    assert_eq!(row.0, 150);

    Ok(())
}

pub struct Sensor {
    pub sensor_id: i32,
    pub sensor_reference: String,
    pub sensor_name: String,
    pub sensor_types: HashSet<SensorType>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SensorType {
    Temperature,
    Humidity,
    Airpressure,
    Co2,
}

pub struct Temperature {
    pub temperature_id: i64,
    pub sensor_id: i32,
    pub temp_celsius: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl Sensor {
    pub fn new_with_id(
        id: i32,
        reference: &str,
        name: &str,
        sensor_types: HashSet<SensorType>,
    ) -> Self {
        Self {
            sensor_id: id,
            sensor_reference: reference.into(),
            sensor_name: name.into(),
            sensor_types,
        }
    }

    pub fn new(reference: &str, name: &str, sensor_types: HashSet<SensorType>) -> Self {
        Self::new_with_id(0, reference, name, sensor_types)
    }
}

impl From<&SensorType> for &str {
    fn from(value: &SensorType) -> Self {
        match value {
            SensorType::Temperature => "temperature",
            SensorType::Humidity => "humidity",
            SensorType::Airpressure => "airpressure",
            SensorType::Co2 => "co2",
        }
    }
}

impl Temperature {
    pub fn new(temp_celsius: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            temperature_id: 0,
            sensor_id: 0,
            temp_celsius,
            measure_time,
        }
    }
}

pub async fn insert_sensor(
    pool: &PgPool,
    sensor: &mut Sensor,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let sensor_id = sqlx::query_scalar!(
        r#"
           INSERT INTO sensors (sensor_name, sensor_reference) VALUES ($1, $2)
           RETURNING sensor_id
        "#,
        sensor.sensor_name,
        sensor.sensor_reference
    )
    .fetch_one(pool)
    .await?;

    sensor.sensor_id = sensor_id;

    Ok(())
}

pub async fn insert_sensor_types(
    pool: &PgPool,
    sensor_id: i32,
    sensor_types: impl Iterator<Item = &SensorType>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let sensor_types_str: Vec<String> = sensor_types
        .map(|st| {
            let st_str: &str = st.into();
            st_str.to_owned()
        })
        .collect();
    sqlx::query!(
        r#"
           INSERT INTO sensor_types_link (sensor_id, sensor_type) 
           SELECT $1, sensor_type_arr.* 
           FROM UNNEST($2::text[]::sensor_type[]) AS sensor_type_arr
        "#,
        sensor_id,
        &sensor_types_str
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_sensor_with_sensor_types(
    pool: &PgPool,
    sensor: &mut Sensor,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    insert_sensor(pool, sensor).await?;
    insert_sensor_types(pool, sensor.sensor_id, sensor.sensor_types.iter()).await?;
    Ok(())
}

pub async fn insert_sensor_temperature_measures(
    pool: &PgPool,
    sensor_id: i32,
    temperatures: &mut Vec<Temperature>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut temps: Vec<BigDecimal> = Vec::with_capacity(temperatures.len());
    let mut times: Vec<chrono::NaiveDateTime> = Vec::with_capacity(temperatures.len());
    for t in temperatures.iter() {
        temps.push(t.temp_celsius.clone());
        times.push(t.measure_time.clone());
    }
    sqlx::query!(
        r#"
           INSERT INTO temperatures (sensor_id, temp_celsius, measure_time) 
           SELECT $1, temperature_arr.* 
           FROM UNNEST($2::numeric(6, 3)[], $3::timestamp[]) AS temperature_arr
        "#,
        sensor_id,
        &temps[..],
        &times[..]
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_sensor_by_ref(
    pool: &PgPool,
    sensor_ref: &str,
) -> std::result::Result<Sensor, Box<dyn std::error::Error>> {
    match sqlx::query!(
        r#"
        SELECT s.sensor_id, s.sensor_name 
        FROM sensors s 
        WHERE s.sensor_reference = $1
    "#,
        sensor_ref
    )
    //TODO: maybe better error handling with anyhow crate?
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(Sensor::new_with_id(
            rec.sensor_id,
            sensor_ref,
            &rec.sensor_name,
            HashSet::new(),
        )),
        Err(e) => Err(e.into()),
    }
}

pub async fn find_sensor_temperature_measures(
    pool: &PgPool,
    sensor_id: i32,
) -> std::result::Result<Vec<Temperature>, Box<dyn std::error::Error>> {
    match sqlx::query_as!(
        Temperature,
        r#"
        SELECT t.temperature_id, s.sensor_id, t.temp_celsius, t.measure_time
        FROM sensors s 
        LEFT JOIN temperatures t ON s.sensor_id = t.sensor_id
        WHERE s.sensor_id = $1
    "#,
        sensor_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(temps) => Ok(temps),
        Err(e) => Err(e.into()),
    }
}
