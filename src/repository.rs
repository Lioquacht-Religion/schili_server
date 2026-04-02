// repository.rs

use std::collections::HashSet;

use chrono::{NaiveDateTime, Utc};
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

pub trait DBSimpleMeasurement {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self;
    fn measurement(&self) -> &BigDecimal;
    fn measure_time(&self) -> NaiveDateTime;
}

pub struct Temperature {
    pub temperature_id: i64,
    pub sensor_id: i32,
    pub temp_celsius: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl DBSimpleMeasurement for Temperature {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.temp_celsius
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
}

pub struct Humidity {
    pub humidity_id: i64,
    pub sensor_id: i32,
    pub humidity_percent: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl DBSimpleMeasurement for Humidity {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.humidity_percent
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
}

pub struct Co2 {
    pub co2_id: i64,
    pub sensor_id: i32,
    pub co2_ppm: BigDecimal,
    pub res0: BigDecimal,
    pub adc_val_12bit: i32,
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
            temperature_id: -1,
            sensor_id: -1,
            temp_celsius,
            measure_time,
        }
    }
}

impl Humidity {
    pub fn new(humidity_percent: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            humidity_id: -1,
            sensor_id: -1,
            humidity_percent,
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

pub async fn insert_single_sensor_temperature(
    pool: &PgPool,
    sensor_id: i32,
    temperature: &mut Temperature,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temperature_id = sqlx::query!(
        r#"
           INSERT INTO temperatures (sensor_id, temp_celsius, measure_time) 
           VALUES ($1, $2, $3)
           RETURNING temperature_id
        "#,
        sensor_id,
        &temperature.temp_celsius,
        &temperature.measure_time
    )
    .fetch_one(pool)
    .await?;

    temperature.temperature_id = temperature_id.temperature_id;

    Ok(())
}

pub async fn insert_sensor_humidity_measures(
    pool: &PgPool,
    sensor_id: i32,
    humidities: &mut Vec<Humidity>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut temps: Vec<BigDecimal> = Vec::with_capacity(humidities.len());
    let mut times: Vec<chrono::NaiveDateTime> = Vec::with_capacity(humidities.len());
    for t in humidities.iter() {
        temps.push(t.humidity_percent.clone());
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

pub async fn insert_single_sensor_humidity(
    pool: &PgPool,
    sensor_id: i32,
    humidity: &mut Humidity,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let rec = sqlx::query!(
        r#"
           INSERT INTO humidities (sensor_id, humidity_percent, measure_time) 
           VALUES ($1, $2, $3)
           RETURNING humidity_id
        "#,
        sensor_id,
        &humidity.humidity_percent,
        &humidity.measure_time
    )
    .fetch_one(pool)
    .await?;

    humidity.humidity_id = rec.humidity_id;

    Ok(())
}

pub async fn find_sensor_humidity_measures_by_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> std::result::Result<Vec<Humidity>, Box<dyn std::error::Error>> {
    match sqlx::query_as!(
        Humidity,
        r#"
        SELECT h.humidity_id, s.sensor_id, h.humidity_percent, h.measure_time
        FROM sensors s 
        LEFT JOIN humidities h ON s.sensor_id = h.sensor_id
        WHERE s.sensor_id = $1
        AND $2 <= h.measure_time AND h.measure_time <= $3
    "#,
        sensor_id,
        start_datetime.naive_utc(),
        end_datetime.naive_utc()
    )
    .fetch_all(pool)
    .await
    {
        Ok(hums) => Ok(hums),
        Err(e) => Err(e.into()),
    }
}

pub async fn insert_single_sensor_co2_measure(
    pool: &PgPool,
    sensor_id: i32,
    co2: &mut Co2,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let co2_id = sqlx::query!(
        r#"
           INSERT INTO co2 (sensor_id, co2_ppm, res0, adc_val_12bit, measure_time) 
           VALUES ($1, $2, $3, $4, $5)
           RETURNING co2_id
        "#,
        sensor_id,
        &co2.co2_ppm,
        &co2.res0,
        &co2.adc_val_12bit,
        &co2.measure_time
    )
    .fetch_one(pool)
    .await?;

    co2.co2_id = co2_id.co2_id;

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

#[deprecated]
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

pub async fn find_sensor_temperature_measures_by_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> std::result::Result<Vec<Temperature>, Box<dyn std::error::Error>> {
    match sqlx::query_as!(
        Temperature,
        r#"
        SELECT t.temperature_id, s.sensor_id, t.temp_celsius, t.measure_time
        FROM sensors s 
        LEFT JOIN temperatures t ON s.sensor_id = t.sensor_id
        WHERE s.sensor_id = $1
        AND $2 <= t.measure_time AND t.measure_time <= $3
    "#,
        sensor_id,
        start_datetime.naive_utc(),
        end_datetime.naive_utc()
    )
    .fetch_all(pool)
    .await
    {
        Ok(temps) => Ok(temps),
        Err(e) => Err(e.into()),
    }
}
