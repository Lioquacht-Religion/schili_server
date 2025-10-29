// repository.rs

use std::collections::HashSet;

use sqlx::{PgPool, Pool, Postgres};

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
    pub id: i32,
    pub reference: String,
    pub name: String,
    pub sensor_types: HashSet<SensorType>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SensorType {
    Temperature,
    Humidity,
    Airpressure,
    Co2,
}

impl Sensor {
    pub fn new(reference: &str, name: &str, sensor_types: HashSet<SensorType>) -> Self {
        Self {
            id: 0,
            reference: reference.into(),
            name: name.into(),
            sensor_types,
        }
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

pub async fn insert_sensor(
    pool: &PgPool,
    sensor: &mut Sensor,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let sensor_id = sqlx::query_scalar!(
        r#"
           INSERT INTO sensors (sensor_name, sensor_reference) VALUES ($1, $2)
           RETURNING sensor_id
        "#,
        sensor.name,
        sensor.reference
    )
    .fetch_one(pool)
    .await?;

    sensor.id = sensor_id;

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
           SELECT $1, table_boo.* 
           FROM UNNEST($2::text[]::sensor_type[]) AS table_boo
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
    insert_sensor_types(pool, sensor.id, sensor.sensor_types.iter()).await?;
    Ok(())
}
