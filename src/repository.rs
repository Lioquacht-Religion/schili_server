// repository.rs

use std::str::FromStr;

use anyhow::{Result, anyhow};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use sqlx::{PgPool, Pool, Postgres, Row, postgres::{PgRow, types::PgInterval}, prelude::FromRow, types::BigDecimal};

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

#[derive(Debug, FromRow)]
pub struct Sensor {
    pub sensor_id: i32,
    pub sensor_reference: String,
    pub sensor_name: String,
    pub sensor_types: Vec<SensorType>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
#[sqlx(type_name = "sensor_type", rename_all = "lowercase")]
pub enum SensorType {
    Temperature,
    Humidity,
    Airpressure,
    LightIntensity,
    Co2,
    BatteryVoltage,
    ChipTemperature,
}

impl<'r> FromRow<'r, PgRow> for SensorType{
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        row.try_get(0)
    }
}

impl From<&SensorType> for &str {
    fn from(value: &SensorType) -> Self {
        match value {
            SensorType::Temperature => "temperature",
            SensorType::Humidity => "humidity",
            SensorType::Airpressure => "airpressure",
            SensorType::LightIntensity => "lightintensity",
            SensorType::ChipTemperature => "chiptemperature",
            SensorType::BatteryVoltage => "BatteryVoltage",
            SensorType::Co2 => "co2",
        }
    }
}

impl FromStr for SensorType{
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
             "temperature"=> Ok(SensorType::Temperature),
             "humidity"=> Ok(SensorType::Humidity),
             "airpressure"=> Ok(SensorType::Airpressure),
             "chiptemperature"=> Ok(SensorType::ChipTemperature),
             "BatteryVoltage"=> Ok(SensorType::BatteryVoltage),
             "co2"=> Ok(SensorType::Co2),
             _ => Err(()),
        }
    }
}

pub trait DBSimpleMeasurement {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self;
    fn measurement(&self) -> &BigDecimal;
    fn measure_time(&self) -> NaiveDateTime;
    fn rounding_places() -> i64;
}

#[derive(Debug, FromRow)]
pub struct SimpleMeasurement {
    pub measurement: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl SimpleMeasurement{
    pub fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self{
        Self { measurement, measure_time }
    }
}

impl DBSimpleMeasurement for SimpleMeasurement{
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.measurement
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
    fn rounding_places() -> i64 {
        18
    }
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
    fn rounding_places() -> i64 {
        3
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
    fn rounding_places() -> i64 {
        3
    }
}

pub struct AirPressure {
    pub air_pressure_id: i64,
    pub sensor_id: i32,
    pub air_pressure_pa: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl DBSimpleMeasurement for AirPressure {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.air_pressure_pa
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
    fn rounding_places() -> i64 {
        3
    }
}

pub struct LightIntensity{
    pub light_intensity_id: i64,
    pub sensor_id: i32,
    pub light_intensity: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl DBSimpleMeasurement for LightIntensity{
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.light_intensity
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
    fn rounding_places() -> i64 {
        9
    }
}

pub struct ChipTemperature {
    pub chip_temperature_id: i64,
    pub sensor_id: i32,
    pub temp_celsius: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl DBSimpleMeasurement for ChipTemperature {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.temp_celsius
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
    fn rounding_places() -> i64 {
        3
    }
}

pub struct BatteryVoltage {
    pub battery_voltage_id: i64,
    pub sensor_id: i32,
    pub battery_volt: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl DBSimpleMeasurement for BatteryVoltage {
    fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self::new(measurement, measure_time)
    }
    fn measurement(&self) -> &BigDecimal {
        &self.battery_volt
    }
    fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
    fn rounding_places() -> i64 {
        6
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
        sensor_types: Vec<SensorType>,
    ) -> Self {
        Self {
            sensor_id: id,
            sensor_reference: reference.into(),
            sensor_name: name.into(),
            sensor_types,
        }
    }

    pub fn new(reference: &str, name: &str, sensor_types: Vec<SensorType>) -> Self {
        Self::new_with_id(0, reference, name, sensor_types)
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

impl AirPressure {
    pub fn new(air_pressure_pa: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            air_pressure_id: -1,
            sensor_id: -1,
            air_pressure_pa,
            measure_time,
        }
    }
}

impl LightIntensity {
    pub fn new(light_intensity: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            light_intensity_id: -1,
            sensor_id: -1,
            light_intensity,
            measure_time,
        }
    }
}



impl ChipTemperature {
    pub fn new(temp_celsius: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            chip_temperature_id: -1,
            sensor_id: -1,
            temp_celsius,
            measure_time,
        }
    }
}

impl BatteryVoltage {
    pub fn new(battery_volt: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            battery_voltage_id: -1,
            sensor_id: -1,
            battery_volt,
            measure_time,
        }
    }
}

// ++++++++++++++ Sensor - SECTION +++++++++++++++++++++

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

pub async fn find_sensor_by_ref(
    pool: &PgPool,
    sensor_ref: &str,
) -> anyhow::Result<Sensor> {
    match sqlx::query!(
        r#"
        SELECT s.sensor_id, s.sensor_name 
        FROM sensors s 
        WHERE s.sensor_reference = $1
    "#,
        sensor_ref
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(Sensor::new_with_id(
            rec.sensor_id,
            sensor_ref,
            &rec.sensor_name,
            Vec::new(),
        )),
        Err(e) => Err(e.into()),
    }
}

pub async fn find_sensor_and_types_by_ref(
    pool: &PgPool,
    sensor_ref: &str,
) -> anyhow::Result<Sensor> {
    let mut sensor = find_sensor_by_ref(pool, sensor_ref).await?;
    let sensor_types: Vec<SensorType> = sqlx::query_as(
        r#"
        SELECT stl.sensor_type
        FROM sensor_types_link stl
        WHERE stl.sensor_id = $1
        ORDER BY stl.sensor_type
    "#
    )
        .bind(sensor.sensor_id)
        .fetch_all(pool)
        .await?;
    sensor.sensor_types = sensor_types.into_iter().collect();
    Ok(sensor)
}

pub async fn find_all_sensors(
    pool: &PgPool,
) -> anyhow::Result<Vec<Sensor>> {
    let sensors = sqlx::query_as(
        r#"
        SELECT s1.sensor_id, s1.sensor_reference, s1.sensor_name, 
            CASE WHEN s2.sensor_types = '{null}'::sensor_type[]
                THEN array[]::sensor_type[]
                ELSE s2.sensor_types
            END AS sensor_types
        FROM sensors s1 
        JOIN (
            SELECT s.sensor_id, array_agg(st.sensor_type) AS sensor_types 
            FROM sensors s 
            LEFT JOIN sensor_types_link st ON s.sensor_id = st.sensor_id 
            GROUP BY s.sensor_id) s2
        ON s1.sensor_id = s2.sensor_id
        "#
    )
        .fetch_all(pool)
        .await?;
    Ok(sensors)
}

pub async fn find_all_sensors_with_filter(
    pool: &PgPool,
    sensor_name_part: &str,
    sensor_types: &[SensorType]
) -> anyhow::Result<Vec<Sensor>> {
    let sensors = sqlx::query_as(
        r#"
        SELECT s1.sensor_id, s1.sensor_reference, s1.sensor_name, 
            s2.sensor_types as "sensor_types"
        FROM sensors s1 
        JOIN (
            SELECT s.sensor_id, coalesce(array_agg(st.sensor_type), 
                array[]::sensor_type[]) AS sensor_types 
            FROM sensors s 
            LEFT JOIN sensor_types_link st ON s.sensor_id = st.sensor_id 
            WHERE s.sensor_name LIKE $1
            GROUP BY s.sensor_id) s2
        ON s1.sensor_id = s2.sensor_id
        AND s2.sensor_types @> $2::sensor_type[]
        "#
    )
        .bind(sensor_name_part)
        .bind(sensor_types)
        .fetch_all(pool)
        .await?;
    Ok(sensors)
}


// ++++++++++++++ Temperature - SECTION +++++++++++++++++++++

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
) -> Result<()> {
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

    const SELECT_IN_TS_RANGE_SQL_1 : &'static str = r#"SELECT m."#;
    const SELECT_IN_TS_RANGE_SQL_2 : &'static str = r#"
         as measurement, m.measure_time as measure_time
        FROM sensors s 
        LEFT JOIN 
        "#;
    const SELECT_IN_TS_RANGE_SQL_3 : &'static str = r#"
         m ON s.sensor_id = m.sensor_id
        WHERE s.sensor_id = $1
        AND $2 <= m.measure_time AND m.measure_time <= $3
        "#;

    const SELECT_TEMP_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(
        SELECT_IN_TS_RANGE_SQL_1, "temp_celsius", SELECT_IN_TS_RANGE_SQL_2, "temperatures", SELECT_IN_TS_RANGE_SQL_3);
    const SELECT_HUM_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_IN_TS_RANGE_SQL_1, "humidity_percent", SELECT_IN_TS_RANGE_SQL_2, "humidities", SELECT_IN_TS_RANGE_SQL_3);
    const SELECT_AIRP_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_IN_TS_RANGE_SQL_1, "air_pressure_pa", SELECT_IN_TS_RANGE_SQL_2, "air_pressures", SELECT_IN_TS_RANGE_SQL_3);
    const SELECT_LIGHTINT_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_IN_TS_RANGE_SQL_1, "light_intensity", SELECT_IN_TS_RANGE_SQL_2, "light_intensities", SELECT_IN_TS_RANGE_SQL_3);
    const SELECT_BATVOLT_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_IN_TS_RANGE_SQL_1, "battery_volt",SELECT_IN_TS_RANGE_SQL_2, "battery_voltages", SELECT_IN_TS_RANGE_SQL_3);
    const SELECT_CHIPTEMP_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_IN_TS_RANGE_SQL_1, "temp_celsius",SELECT_IN_TS_RANGE_SQL_2, "chip_temperatures", SELECT_IN_TS_RANGE_SQL_3);

pub async fn find_sensor_temperatures_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    find_sensor_simple_measures_in_timerange(
        pool, sensor_id, start_datetime, end_datetime,
        SELECT_TEMP_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_humidities_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    find_sensor_simple_measures_in_timerange(
        pool, sensor_id, start_datetime, end_datetime,
        SELECT_HUM_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_airpressures_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    find_sensor_simple_measures_in_timerange(
        pool, sensor_id, start_datetime, end_datetime,
        SELECT_AIRP_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_lightintensities_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    find_sensor_simple_measures_in_timerange(
        pool, sensor_id, start_datetime, end_datetime,
        SELECT_LIGHTINT_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_batteryvolt_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    find_sensor_simple_measures_in_timerange(
        pool, sensor_id, start_datetime, end_datetime,
        SELECT_BATVOLT_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_chiptemperature_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    find_sensor_simple_measures_in_timerange(
        pool, sensor_id, start_datetime, end_datetime,
        SELECT_CHIPTEMP_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_simple_measures_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    select_in_ts: &'static str,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    match sqlx::query_as::<_, SimpleMeasurement>(select_in_ts)
        .bind(sensor_id)
        .bind(start_datetime)
        .bind(end_datetime)
    .fetch_all(pool)
    .await
    {
        Ok(temps) => Ok(temps),
        Err(e) => Err(e.into()),
    }
}

    const MIN_MAX_SQL_1 : &'static str = "
        SELECT MIN(t.measure_time) min_ts, MAX(t.measure_time) max_ts
        FROM 
        ";
    const MIN_MAX_SQL_2 : &'static str = "
         t WHERE t.sensor_id = $1
         AND t.measure_time >= $2
         AND t.measure_time <= $3
        ";

    const SELECT_MIN_MAX_TEMP_SQL : &'static str = const_format::concatcp!(MIN_MAX_SQL_1, "temperatures", MIN_MAX_SQL_2);
    const SELECT_MIN_MAX_HUM_SQL: &'static str = const_format::concatcp!(MIN_MAX_SQL_1, "humidities", MIN_MAX_SQL_2);
    const SELECT_MIN_MAX_AIRP_SQL: &'static str = const_format::concatcp!(MIN_MAX_SQL_1, "air_pressures", MIN_MAX_SQL_2);
    const SELECT_MIN_MAX_LIGHTINT_SQL: &'static str = const_format::concatcp!(MIN_MAX_SQL_1, "light_intensities", MIN_MAX_SQL_2);
    const SELECT_MIN_MAX_BATVOLT_SQL: &'static str = const_format::concatcp!(MIN_MAX_SQL_1, "battery_voltages", MIN_MAX_SQL_2);
    const SELECT_MIN_MAX_CHIPTEMP_SQL: &'static str = const_format::concatcp!(MIN_MAX_SQL_1, "chip_temperatures", MIN_MAX_SQL_2);


    const SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1 : &'static str = "
        SELECT d::timestamp timestamp_from, avg(m.";

    const SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2 : &'static str = "
        ) avg_measurement
        FROM ";
    const SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3 : &'static str = "
         m
        INNER JOIN generate_series(
            $2::timestamp, $3::timestamp, $4::interval
        ) d
        ON m.measure_time >= d::timestamp AND m.measure_time <= d::timestamp + $4 
        WHERE 
            m.sensor_id = $1
            AND m.measure_time >= $2 AND m.measure_time <= $3
        GROUP BY d::timestamp ORDER BY d::timestamp desc
        ";

    const SELECT_AVG_TEMP_INTERVALS_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(
        SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1, "temp_celsius", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2, "temperatures", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3);
    const SELECT_AVG_HUM_INTERVALS_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1, "humidity_percent", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2, "humidities", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3);
    const SELECT_AVG_AIRP_INTERVALS_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1, "air_pressure_pa", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2, "air_pressures", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3, );
    const SELECT_AVG_LIGHTINT_INTERVALS_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1, "light_intensity", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2, "light_intensities", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3, );
    const SELECT_AVG_BATVOLT_INTERVALS_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1, "battery_volt",SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2, "battery_voltages", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3);
    const SELECT_AVG_CHIPTEMP_INTERVALS_IN_TS_RANGE_SQL : &'static str = const_format::concatcp!(SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_1, "temp_celsius",SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_2, "chip_temperatures", SELECT_AVG_INTERVALS_IN_TS_RANGE_SQL_3);

#[derive(FromRow)]
pub struct MinTsMaxTs{
    min_ts: Option<NaiveDateTime>,
    max_ts: Option<NaiveDateTime>
}

#[derive(FromRow)]
pub struct AvgMeasureTimeInterval{
    pub timestamp_from: NaiveDateTime,
    pub avg_measurement: BigDecimal
}

const MAX_INTERVAL_NUM: u64= 10_000;

pub async fn find_sensor_avg_temperatures_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    find_sensor_avg_simple_measures_by_intervals_in_timerange(
        pool, sensor_id, start_datetime, end_datetime, interval, 
        SELECT_MIN_MAX_TEMP_SQL, 
        SELECT_AVG_TEMP_INTERVALS_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_avg_humidities_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    find_sensor_avg_simple_measures_by_intervals_in_timerange(
        pool, sensor_id, start_datetime, end_datetime, interval, 
        SELECT_MIN_MAX_HUM_SQL, 
        SELECT_AVG_HUM_INTERVALS_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_avg_airpressures_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    find_sensor_avg_simple_measures_by_intervals_in_timerange(
        pool, sensor_id, start_datetime, end_datetime, interval, 
        SELECT_MIN_MAX_AIRP_SQL, 
        SELECT_AVG_AIRP_INTERVALS_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_avg_lightintensities_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    find_sensor_avg_simple_measures_by_intervals_in_timerange(
        pool, sensor_id, start_datetime, end_datetime, interval, 
        SELECT_MIN_MAX_LIGHTINT_SQL, 
        SELECT_AVG_LIGHTINT_INTERVALS_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_avg_battvolt_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    find_sensor_avg_simple_measures_by_intervals_in_timerange(
        pool, sensor_id, start_datetime, end_datetime, interval, 
        SELECT_MIN_MAX_BATVOLT_SQL, 
        SELECT_AVG_BATVOLT_INTERVALS_IN_TS_RANGE_SQL
    ).await
}

pub async fn find_sensor_avg_chip_temperature_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    find_sensor_avg_simple_measures_by_intervals_in_timerange(
        pool, sensor_id, start_datetime, end_datetime, interval, 
        SELECT_MIN_MAX_CHIPTEMP_SQL, 
        SELECT_AVG_CHIPTEMP_INTERVALS_IN_TS_RANGE_SQL
    ).await
}

//TODO: improve performance, add safe guards, max min start end datetimes
//TODO: add better errors with specific enum
pub async fn find_sensor_avg_simple_measures_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta,
    min_max_ts_sql: &'static str,
    select_interval_in_ts: &'static str
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    let interval_mill_secs = interval.num_milliseconds();
    if interval_mill_secs == 0 {
        return Err(anyhow!("Interval cannot be zero."));
    }
    let interval_count = ((*end_datetime - start_datetime).num_milliseconds() / interval.num_milliseconds()).unsigned_abs();
    if interval_count > MAX_INTERVAL_NUM{
        return Err(anyhow!("Number of intervals between start and end timestamp is above maximum of 10.000. Interval: {interval_count}"));
    }
    let interval : PgInterval = interval.try_into()
        .map_err(|e| anyhow!("Invalid interval was supplied: {}", e))?;

    let min_max_ts  = sqlx::query_as::<_, MinTsMaxTs>(min_max_ts_sql)
        .bind(sensor_id)
        .bind(start_datetime)
        .bind(end_datetime)
    .fetch_one(pool).await?;

    let (start_datetime, end_datetime) = 
    if let (Some(min_ts), Some(max_ts)) = (min_max_ts.min_ts, min_max_ts.max_ts){
        (
            start_datetime.naive_utc().clamp(min_ts, max_ts), 
            end_datetime.naive_utc().clamp(min_ts, max_ts),
        )
    }
    else{
        return Err(anyhow!("No entries found!"));
    };

    match sqlx::query_as::<_, AvgMeasureTimeInterval>(select_interval_in_ts)
        .bind(sensor_id)
        .bind(start_datetime)
        .bind(end_datetime)
        .bind(interval)
    .fetch_all(pool)
    .await
    {
        Ok(temps) => Ok(temps),
        Err(e) => Err(e.into()),
    }
}

pub async fn find_sensor_last_temperature_before_at_datetime(
    pool: &PgPool,
    sensor_id: i32,
    before_at_datetime: &chrono::DateTime<Utc>,
) -> std::result::Result<Temperature, Box<dyn std::error::Error>> {
    match sqlx::query_as!(
        Temperature,
        r#"
        SELECT t.temperature_id, t.sensor_id, t.temp_celsius, t.measure_time
        FROM temperatures t 
        WHERE t.sensor_id = $1
        AND t.measure_time <= $2
        ORDER BY t.measure_time DESC LIMIT 1
        "#,
        sensor_id,
        before_at_datetime.naive_utc(),
    )
    .fetch_one(pool)
    .await
    {
        Ok(temps) => Ok(temps),
        Err(e) => Err(e.into()),
    }
}

// ++++++++++++++ Humidity - SECTION +++++++++++++++++++++

pub async fn insert_sensor_humidity_measures(
    pool: &PgPool,
    sensor_id: i32,
    humidities: &mut Vec<Humidity>,
) -> Result<()> {
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
) -> Result<()> {
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

// ++++++++++++++ Airpressure - SECTION +++++++++++++++++++++

pub async fn insert_sensor_airpressure_measures(
    pool: &PgPool,
    sensor_id: i32,
    airpressures: &mut Vec<AirPressure>,
) -> Result<()> {
    let mut temps: Vec<BigDecimal> = Vec::with_capacity(airpressures.len());
    let mut times: Vec<chrono::NaiveDateTime> = Vec::with_capacity(airpressures.len());
    for t in airpressures.iter() {
        temps.push(t.air_pressure_pa.clone());
        times.push(t.measure_time.clone());
    }
    sqlx::query!(
        r#"
           INSERT INTO air_pressures (sensor_id, air_pressure_pa, measure_time) 
           SELECT $1, air_pressures_arr.* 
           FROM UNNEST($2::numeric(6, 3)[], $3::timestamp[]) AS air_pressures_arr
        "#,
        sensor_id,
        &temps[..],
        &times[..]
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_single_sensor_airpressure(
    pool: &PgPool,
    sensor_id: i32,
    air_pressure: &mut AirPressure,
) -> Result<()> {
    let rec = sqlx::query!(
        r#"
           INSERT INTO air_pressures (sensor_id, air_pressure_pa, measure_time) 
           VALUES ($1, $2, $3)
           RETURNING air_pressure_id
        "#,
        sensor_id,
        &air_pressure.air_pressure_pa,
        &air_pressure.measure_time
    )
    .fetch_one(pool)
    .await?;

    air_pressure.air_pressure_id = rec.air_pressure_id;

    Ok(())
}

// ++++++++++++++ Lightintesity - SECTION +++++++++++++++++++++

pub async fn insert_single_sensor_lightintensity(
    pool: &PgPool,
    sensor_id: i32,
    light_intensity: &mut LightIntensity,
) -> Result<()> {
    let rec = sqlx::query!(
        r#"
           INSERT INTO light_intensities (sensor_id, light_intensity, measure_time) 
           VALUES ($1, $2, $3)
           RETURNING light_intensity_id
        "#,
        sensor_id,
        &light_intensity.light_intensity,
        &light_intensity.measure_time
    )
    .fetch_one(pool)
    .await?;

    light_intensity.light_intensity_id = rec.light_intensity_id;

    Ok(())
}

// ++++++++++++++ Chip temerature - SECTION +++++++++++++++++++++

pub async fn insert_single_sensor_chip_temperature(
    pool: &PgPool,
    sensor_id: i32,
    chip_temp: &mut ChipTemperature,
) -> Result<()> {
    let rec = sqlx::query!(
        r#"
           INSERT INTO chip_temperatures (sensor_id, temp_celsius, measure_time) 
           VALUES ($1, $2, $3)
           RETURNING chip_temperature_id
        "#,
        sensor_id,
        &chip_temp.temp_celsius,
        &chip_temp.measure_time
    )
    .fetch_one(pool)
    .await?;

    chip_temp.chip_temperature_id = rec.chip_temperature_id;

    Ok(())
}

// ++++++++++++++ Battery voltage - SECTION +++++++++++++++++++++

pub async fn insert_single_sensor_battery_voltage(
    pool: &PgPool,
    sensor_id: i32,
    batt_volt: &mut BatteryVoltage,
) -> Result<()> {
    let rec = sqlx::query!(
        r#"
           INSERT INTO battery_voltages (sensor_id, battery_volt, measure_time) 
           VALUES ($1, $2, $3)
           RETURNING battery_voltage_id
        "#,
        sensor_id,
        &batt_volt.battery_volt,
        &batt_volt.measure_time
    )
    .fetch_one(pool)
    .await?;

    batt_volt.battery_voltage_id = rec.battery_voltage_id;

    Ok(())
}

// ++++++++++++++ Co2 - SECTION +++++++++++++++++++++

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

// ++++++++++++++ Error - SECTION +++++++++++++++++++++

pub struct SensorError{
    pub sensor_error_id: i64,
    pub sensor_id: i32,
    pub error_code: i32,
    pub error_text: String,
    pub error_time: NaiveDateTime,
}

pub async fn insert_single_sensor_error(
    pool: &PgPool,
    sensor_id: i32,
    error: &mut SensorError,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let sensor_error_id = sqlx::query!(
        r#"
           INSERT INTO sensor_errors (sensor_id, error_code, error_text, error_time) 
           VALUES ($1, $2, $3, $4)
           RETURNING sensor_error_id
        "#,
        sensor_id,
        &error.error_code,
        &error.error_text,
        &error.error_time
    )
    .fetch_one(pool)
    .await?;

    error.sensor_error_id = sensor_error_id.sensor_error_id;

    Ok(())
}


