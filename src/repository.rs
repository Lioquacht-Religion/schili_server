// repository.rs

use std::str::FromStr;

use anyhow::{Result, anyhow};
use chrono::{Local, NaiveDateTime, TimeDelta, Utc};
use schili_api::api::{self, MeasureLimit, SensorType, SensorTypeWarnLimits};
use sqlx::{
    PgPool, Postgres, Row, postgres::{PgRow, PgValueRef, types::PgInterval}, prelude::FromRow, types::BigDecimal,
};

use crate::api_db_conv::{ModelFrom, ModelInto};

#[derive(Debug, FromRow)]
pub struct Sensor {
    pub sensor_id: i32,
    pub sensor_reference: String,
    pub sensor_name: String,
    pub sensor_types: Vec<SensorTypeDB>,
}

pub struct SensorTypeWarnLimitsDAO{
     sensor_type: SensorTypeDB,
     warn_high_num: Option<BigDecimal>,
     warn_high_timeout: Option<PgInterval>,
     warn_low_num: Option<BigDecimal>,
     warn_low_timeout: Option<PgInterval>,
     warn_increase_num: Option<BigDecimal>,
     warn_increase_search_interval: Option<PgInterval>,
     warn_decrease_num: Option<BigDecimal>,
     warn_decrease_search_interval: Option<PgInterval>,
}

fn measurelimits_from_dao(num: Option<BigDecimal>, interval: Option<PgInterval>) -> Option<MeasureLimit> {
    if let (Some(num), Some(interval)) = (num, interval){
        let dur = chrono::Duration::microseconds(interval.microseconds);
        Some(MeasureLimit::new(num, dur))
    }
    else{
        None
    }
}

fn pg_interval_from_dur(duration: chrono::Duration) -> PgInterval{
    PgInterval { 
        months: 0, days: 0, microseconds: duration.num_microseconds().unwrap_or(0) 
    }
}

impl From<SensorTypeWarnLimitsDAO> for SensorTypeWarnLimits{
    fn from(value: SensorTypeWarnLimitsDAO) -> Self {
        Self {
            sensor_type: value.sensor_type.model_into(),
            warn_high: measurelimits_from_dao(value.warn_high_num, value.warn_high_timeout),
            warn_low: measurelimits_from_dao(value.warn_low_num, value.warn_low_timeout),
            warn_increase: measurelimits_from_dao(value.warn_increase_num, value.warn_increase_search_interval),
            warn_decrease: measurelimits_from_dao(value.warn_decrease_num, value.warn_decrease_search_interval),
        }
    }
}

impl ModelFrom<&SensorTypeWarnLimits> for SensorTypeWarnLimitsDAO{
    fn model_from(value: &SensorTypeWarnLimits) -> Self {
        Self {
            sensor_type: value.sensor_type.into(),
            warn_high_num: value.warn_high.as_ref().map(|n| n.limit.clone()),
            warn_high_timeout: value.warn_high.as_ref().map(|n| pg_interval_from_dur(n.time_duration)),
            warn_low_num: value.warn_low.as_ref().map(|n| n.limit.clone()),
            warn_low_timeout: value.warn_low.as_ref().map(|n| pg_interval_from_dur(n.time_duration)),
            warn_increase_num: value.warn_increase.as_ref().map(|n| n.limit.clone()),
            warn_increase_search_interval: value.warn_increase.as_ref().map(|n| pg_interval_from_dur(n.time_duration)),
            warn_decrease_num: value.warn_decrease.as_ref().map(|n| n.limit.clone()),
            warn_decrease_search_interval: value.warn_decrease.as_ref().map(|n| pg_interval_from_dur(n.time_duration)),
        }
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorTypeDB(api::SensorType);

impl SensorTypeDB{
    pub fn inner(&self) -> api::SensorType{
        self.0
    }
}

impl From<SensorType> for SensorTypeDB{
    fn from(value: SensorType) -> Self {
        SensorTypeDB(value)
    }
}

impl ModelFrom<SensorTypeDB> for SensorType{
    fn model_from(value: SensorTypeDB) -> Self {
        value.0
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for SensorTypeDB{
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&'r str as sqlx::decode::Decode<'r, Postgres>>::decode(value)?;
        match SensorType::from_str(value){
            Ok(st) => Ok(st.into()),
            Err(()) => Err(
                format!("invalid value {0:?} for enum {1}", value, "SensorType").into()
            ),
        }
    }
}

impl<'q, DB: ::sqlx::Database> ::sqlx::encode::Encode<'q, DB> for SensorTypeDB
where
    &'q ::std::primitive::str: ::sqlx::encode::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as ::sqlx::database::Database>::ArgumentBuffer<'q>,
    ) -> ::std::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
        let val: &str = (&self.0).into();
        val.encode(buf)
    }
    fn size_hint(&self) -> ::std::primitive::usize {
        let val: &str = (&self.0).into();
        val.size_hint()
    }
}

impl ::sqlx::Type<::sqlx::Postgres> for SensorTypeDB {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        ::sqlx::postgres::PgTypeInfo::with_name("sensor_type")
    }
}

impl ::sqlx::postgres::PgHasArrayType for SensorTypeDB {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        ::sqlx::postgres::PgTypeInfo::array_of("sensor_type")
    }
}

impl<'r> FromRow<'r, PgRow> for SensorTypeDB {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        row.try_get(0)
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

impl SimpleMeasurement {
    pub fn new(measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self {
            measurement,
            measure_time,
        }
    }
}

impl DBSimpleMeasurement for SimpleMeasurement {
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

//TODO: replace current models with new generic one
pub struct SensorMeasurement{
    pub sensor_measurement_id: i64,
    pub sensor_id: i32,
    pub measure_type: SensorTypeDB,
    pub measurement: BigDecimal,
    pub measure_time: NaiveDateTime,
}

impl SensorMeasurement {
    pub fn new(sensor_type: SensorTypeDB, measurement: BigDecimal, measure_time: NaiveDateTime) -> Self {
        Self{
            sensor_id: -1,
            sensor_measurement_id: -1,
            measure_type: sensor_type,
            measurement,
            measure_time}
    }
    pub fn measurement(&self) -> &BigDecimal {
        &self.measurement
    }
    pub fn measure_time(&self) -> NaiveDateTime {
        self.measure_time
    }
    pub fn rounding_places() -> i64 {
        9
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
        sensor_types: Vec<SensorTypeDB>,
    ) -> Self {
        Self {
            sensor_id: id,
            sensor_reference: reference.into(),
            sensor_name: name.into(),
            sensor_types,
        }
    }

    pub fn new(reference: &str, name: &str, sensor_types: Vec<SensorTypeDB>) -> Self {
        Self::new_with_id(0, reference, name, sensor_types)
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
    sensor_types: &[SensorTypeWarnLimitsDAO],
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let sensor_types_str: Vec<String> = sensor_types.iter()
        .map(|st| {
            let st_str: &str = (&st.sensor_type.0).into();
            st_str.to_owned()
        })
        .collect();

    fn warn_limits_as_vecs(
        sensor_types: &[SensorTypeWarnLimitsDAO],
        get_limit: impl Fn(&SensorTypeWarnLimitsDAO) -> &Option<BigDecimal>,
        get_timeout: impl Fn(&SensorTypeWarnLimitsDAO) -> &Option<PgInterval>
    ) -> (Vec<Option<BigDecimal>>, Vec<Option<PgInterval>>){
        let warn_limits: Vec<Option<BigDecimal>> = sensor_types.iter()
            .map(|st| get_limit(st).clone())
            .collect();
        let warn_timeout: Vec<Option<PgInterval>> = sensor_types.iter()
            .map(|st| get_timeout(st).clone())
            .collect();
        (warn_limits, warn_timeout)
    }

    let (warn_high_num, warn_high_timeout) = warn_limits_as_vecs(sensor_types, |st| &st.warn_high_num, |st| &st.warn_high_timeout);

    let (warn_low_num, warn_low_timeout) = warn_limits_as_vecs(sensor_types, |st| &st.warn_low_num, |st| &st.warn_low_timeout);

    let (warn_increase_num, warn_increase_timeout) = warn_limits_as_vecs(sensor_types, |st| &st.warn_increase_num, |st| &st.warn_increase_search_interval);

    let (warn_decrease_num, warn_decrease_timeout) = warn_limits_as_vecs(sensor_types, |st| &st.warn_decrease_num, |st| &st.warn_decrease_search_interval);

    sqlx::query!(
        r#"
           INSERT INTO sensor_types_link (sensor_id, sensor_type) 
           SELECT $1, sensor_type_arr.* 
           FROM 
            UNNEST($2::text[]::sensor_type[]) AS sensor_type_arr,
            UNNEST($3::numeric[]) AS warn_high_num_arr,
            UNNEST($4::interval[]) AS warn_high_timeout_arr,
            UNNEST($5::numeric[]) AS warn_low_num_arr,
            UNNEST($6::interval[]) AS warn_low_timeout_arr,
            UNNEST($7::numeric[]) AS warn_increase_num_arr,
            UNNEST($8::interval[]) AS warn_increase_timeout_arr,
            UNNEST($9::numeric[]) AS warn_decrease_num_arr,
            UNNEST($10::interval[]) AS warn_decrease_timeout_arr
        "#,
        sensor_id,
        &sensor_types_str,
        &warn_high_num as _,
        &warn_high_timeout as _,
        &warn_low_num as _,
        &warn_low_timeout as _,
        &warn_increase_num as _,
        &warn_increase_timeout as _,
        &warn_decrease_num as _,
        &warn_decrease_timeout as _,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_sensor_with_sensor_types(
    pool: &PgPool,
    sensor: &mut Sensor,
    warn_limits: &[SensorTypeWarnLimitsDAO]
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    insert_sensor(pool, sensor).await?;
    insert_sensor_types(
        pool, 
        sensor.sensor_id, 
        warn_limits
    ).await?;
    Ok(())
}

pub async fn find_sensor_by_ref(pool: &PgPool, sensor_ref: &str) -> anyhow::Result<Sensor> {
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
    let sensor_types: Vec<SensorTypeDB> = sqlx::query_as(
        r#"
        SELECT stl.sensor_type
        FROM sensor_types_link stl
        WHERE stl.sensor_id = $1
        ORDER BY stl.sensor_type
    "#,
    )
    .bind(sensor.sensor_id)
    .fetch_all(pool)
    .await?;
    sensor.sensor_types = sensor_types.into_iter().collect();
    Ok(sensor)
}

pub async fn find_all_sensors(pool: &PgPool) -> anyhow::Result<Vec<Sensor>> {
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
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(sensors)
}

pub async fn find_all_sensors_with_filter(
    pool: &PgPool,
    sensor_name_part: &str,
    sensor_types: &[SensorTypeDB],
) -> anyhow::Result<Vec<Sensor>> {
    let sensors = sqlx::query_as(
        r#"
        SELECT s1.sensor_id, s1.sensor_reference, s1.sensor_name, 
            s2.sensor_types
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
        "#,
    )
    .bind(sensor_name_part)
    .bind(sensor_types)
    .fetch_all(pool)
    .await?;
    Ok(sensors)
}

pub async fn find_sensor_with_warnlimits(
    pool: &PgPool,
    sensor_ref: &str
) -> anyhow::Result<(Sensor, Vec<SensorTypeWarnLimits>)> {
    let sensor = find_sensor_by_ref(pool, sensor_ref).await
        .map_err(|e| 
            anyhow!("Could not find sensor with reference='{}'. Error: {e}", sensor_ref))?;
    let warn_limits = find_sensor_warnlimits(pool, sensor.sensor_id)
        .await
        .map_err(|e| anyhow!("Error while trying to fetch sensor warn limits: {e}"))?;
    Ok((sensor, warn_limits))
}

pub async fn find_sensor_warnlimits(
    pool: &PgPool,
    sensor_id: i32
) -> anyhow::Result<Vec<SensorTypeWarnLimits>> {
    let results = sqlx::query_as!(
        SensorTypeWarnLimitsDAO,
        r#"
        SELECT 
            st.sensor_type::sensor_type as "sensor_type: SensorTypeDB",
            st.warn_high_num, st.warn_high_timeout,
            st.warn_low_num, st.warn_low_timeout, 
            st.warn_increase_num, st.warn_increase_search_interval, 
            st.warn_decrease_num, st.warn_decrease_search_interval
        FROM sensor_types_link st 
        WHERE st.sensor_id = $1
        "#,
        sensor_id
    )
        .map(|s| s.into())
    .fetch_all(pool)
    .await?;

    Ok(results)
}

// ++++++++++++++ Measurements - SECTION +++++++++++++++++++++

pub async fn insert_single_sensor_measurement(
    pool: &PgPool,
    sensor_id: i32,
    measurement: &mut SensorMeasurement,
) -> Result<()> {
    let measurement_id= sqlx::query!(
        r#"
           INSERT INTO sensor_measurements (sensor_id, measure_type, measurement, measure_time) 
           VALUES ($1, $2, $3, $4)
           RETURNING sensor_measurement_id
        "#,
        sensor_id,
        measurement.measure_type as _,
        &&measurement.measurement,
        &measurement.measure_time
    )
    .fetch_one(pool)
    .await?;

    measurement.sensor_measurement_id = measurement_id.sensor_measurement_id;

    Ok(())
}

pub(crate) async fn insert_sensor_measurements(
    pool: &PgPool,
    sensor_id: i32,
    measurements: &mut Vec<SensorMeasurement>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut sensor_types: Vec<SensorTypeDB> = Vec::with_capacity(measurements.len());
    let mut temps: Vec<BigDecimal> = Vec::with_capacity(measurements.len());
    let mut times: Vec<chrono::NaiveDateTime> = Vec::with_capacity(measurements.len());
    for t in measurements.iter() {
        sensor_types.push(t.measure_type);
        temps.push(t.measurement.clone());
        times.push(t.measure_time.clone());
    }
    sqlx::query!(
        r#"
        INSERT INTO sensor_measurements (sensor_id, measure_type, measurement, measure_time) 
        SELECT $1, measurement_arr.* 
        FROM UNNEST(
            $2::sensor_type[], $3::numeric(6, 3)[], $4::timestamp[]
        ) AS measurement_arr
        "#,
        sensor_id,
        &sensor_types[..] as _,
        &temps[..],
        &times[..]
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_sensor_measurements_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    sensor_type: SensorTypeDB,
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
) -> anyhow::Result<Vec<SimpleMeasurement>> {
    match sqlx::query_as!(
        SimpleMeasurement,
        r#"SELECT m.measurement, m.measure_time
        FROM sensors s 
        LEFT JOIN 
         sensor_measurements m ON s.sensor_id = m.sensor_id
        WHERE s.sensor_id = $1
        AND m.measure_type = $2
        AND $3 <= m.measure_time AND m.measure_time <= $4
        "#,
        sensor_id,
        sensor_type as _,
        start_datetime.naive_local(),
        end_datetime.naive_local()
        )
        .fetch_all(pool)
        .await
    {
        Ok(measurements) => Ok(measurements),
        Err(e) => Err(e.into()),
    }
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
        Ok(measurements) => Ok(measurements),
        Err(e) => Err(e.into()),
    }
}

#[derive(FromRow)]
pub struct MinTsMaxTs {
    min_ts: Option<NaiveDateTime>,
    max_ts: Option<NaiveDateTime>,
}

#[derive(FromRow)]
pub struct AvgMeasureTimeInterval {
    pub timestamp_from: NaiveDateTime,
    pub avg_measurement: BigDecimal,
}

const MAX_INTERVAL_NUM: u64 = 10_000;

pub async fn find_sensor_avg_measurements_by_intervals_in_timerange(
    pool: &PgPool,
    sensor_id: i32,
    sensor_type: SensorTypeDB, 
    start_datetime: &chrono::DateTime<Utc>,
    end_datetime: &chrono::DateTime<Utc>,
    interval: TimeDelta,
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    let interval_mill_secs = interval.num_milliseconds();
    if interval_mill_secs == 0 {
        return Err(anyhow!("Interval cannot be zero."));
    }
    let interval_count = ((*end_datetime - start_datetime).num_milliseconds()
        / interval.num_milliseconds())
    .unsigned_abs();
    if interval_count > MAX_INTERVAL_NUM {
        return Err(anyhow!(
            "Number of intervals between start and end timestamp is above maximum of 10.000. Interval: {interval_count}"
        ));
    }
    let interval: PgInterval = interval
        .try_into()
        .map_err(|e| anyhow!("Invalid interval was supplied: {}", e))?;

    let min_max_ts = sqlx::query_as!(MinTsMaxTs,
        "
        SELECT MIN(m.measure_time) min_ts, MAX(m.measure_time) max_ts
        FROM sensor_measurements m 
        WHERE m.sensor_id = $1
         AND m.measure_type = $2
         AND m.measure_time >= $3 AND m.measure_time <= $4
        ",
        sensor_id,
        sensor_type as _,
        start_datetime.naive_local(),
        end_datetime.naive_local()
        )
        .fetch_one(pool)
        .await?;

    let (start_datetime, end_datetime) =
        if let (Some(min_ts), Some(max_ts)) = (min_max_ts.min_ts, min_max_ts.max_ts) {
            (
                start_datetime.naive_utc().clamp(min_ts, max_ts),
                end_datetime.naive_utc().clamp(min_ts, max_ts),
            )
        } else {
            return Err(anyhow!("No entries found!"));
        };

    match sqlx::query_as!(
        AvgMeasureTimeInterval,
    r#"
        SELECT 
            d::timestamp as "timestamp_from!", 
            avg(m.measurement) as "avg_measurement!"
        FROM 
            sensor_measurements m
        INNER JOIN generate_series(
            $3::timestamp, $4::timestamp, $5::interval
        ) d
        ON m.measure_time >= d::timestamp AND m.measure_time <= d::timestamp + $5 
        WHERE 
            m.sensor_id = $1
            AND m.measure_type = $2
            AND m.measure_time >= $3 AND m.measure_time <= $4
        GROUP BY d::timestamp ORDER BY d::timestamp desc
        "#,
        sensor_id,
        sensor_type as _,
        start_datetime,
        end_datetime,
        interval
        )
        .fetch_all(pool)
        .await
    {
        Ok(temps) => Ok(temps),
        Err(e) => Err(e.into()),
    }
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
    select_interval_in_ts: &'static str,
) -> anyhow::Result<Vec<AvgMeasureTimeInterval>> {
    let interval_mill_secs = interval.num_milliseconds();
    if interval_mill_secs == 0 {
        return Err(anyhow!("Interval cannot be zero."));
    }
    let interval_count = ((*end_datetime - start_datetime).num_milliseconds()
        / interval.num_milliseconds())
    .unsigned_abs();
    if interval_count > MAX_INTERVAL_NUM {
        return Err(anyhow!(
            "Number of intervals between start and end timestamp is above maximum of 10.000. Interval: {interval_count}"
        ));
    }
    let interval: PgInterval = interval
        .try_into()
        .map_err(|e| anyhow!("Invalid interval was supplied: {}", e))?;

    let min_max_ts = sqlx::query_as::<_, MinTsMaxTs>(min_max_ts_sql)
        .bind(sensor_id)
        .bind(start_datetime)
        .bind(end_datetime)
        .fetch_one(pool)
        .await?;

    let (start_datetime, end_datetime) =
        if let (Some(min_ts), Some(max_ts)) = (min_max_ts.min_ts, min_max_ts.max_ts) {
            (
                start_datetime.naive_utc().clamp(min_ts, max_ts),
                end_datetime.naive_utc().clamp(min_ts, max_ts),
            )
        } else {
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

pub(crate) async fn find_sensor_last_measurement_before_at_datetime(
    pool: &PgPool,
    sensor_id: i32,
    sensor_type: SensorTypeDB,
    before_at_datetime: &chrono::DateTime<Local>,
) -> std::result::Result<SimpleMeasurement, Box<dyn std::error::Error>> {
    match sqlx::query_as!(
        SimpleMeasurement,
        r#"
        SELECT t.measurement, t.measure_time
        FROM sensor_measurements t 
        WHERE t.sensor_id = $1
            AND t.measure_type = $2
            AND t.measure_time <= $3
        ORDER BY t.measure_time DESC LIMIT 1
        "#,
        sensor_id,
        sensor_type as _,
        before_at_datetime.naive_local(),
    )
    .fetch_one(pool)
    .await
    {
        Ok(measurements) => Ok(measurements),
        Err(e) => Err(e.into()),
    }
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

pub struct SensorError {
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
