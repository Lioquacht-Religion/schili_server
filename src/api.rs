// api.rs

use std::collections::HashSet;

use chrono::{Utc, serde::ts_seconds};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Sensor {
    pub reference: String,
    pub name: String,
    pub sensor_types: HashSet<SensorType>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SensorType {
    Temperature,
    Humidity,
    Airpressure,
    Co2,
}

#[derive(Deserialize, Serialize)]
pub struct PostTemperature {
    pub sensor_reference: String,
    pub temp_celsius: f32,
    #[serde(with = "ts_seconds")]
    pub measure_time: chrono::DateTime<Utc>,
}

impl Sensor {
    pub fn new(reference: &str, name: &str, sensor_types: HashSet<SensorType>) -> Self {
        Self {
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
