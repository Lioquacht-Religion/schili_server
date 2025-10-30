// api_db_conv.rs

use std::collections::HashSet;

use crate::{api::{self, SensorTempMeasurements}, repository};

impl From<&api::SensorType> for repository::SensorType {
    fn from(value: &api::SensorType) -> Self {
        match value {
            api::SensorType::Temperature => repository::SensorType::Temperature,
            api::SensorType::Humidity => repository::SensorType::Humidity,
            api::SensorType::Airpressure => repository::SensorType::Airpressure,
            api::SensorType::Co2 => repository::SensorType::Co2,
        }
    }
}

impl From<&api::Sensor> for repository::Sensor {
    fn from(value: &api::Sensor) -> Self {
        let sensor_types: HashSet<repository::SensorType> =
            value.sensor_types.iter().map(|st| st.into()).collect();
        repository::Sensor::new(&value.reference, &value.name, sensor_types)
    }
}

impl From<&api::SensorTempMeasurements> for (String, Vec<repository::Temperature>) {
    fn from(value: &api::SensorTempMeasurements) -> Self {
        let temps: Vec<repository::Temperature> = value
            .temp_measurements
            .iter()
            .map(|t| {
                repository::Temperature::new(
                    t.temp_celsius.clone(),
                    t.measure_time.naive_utc().clone(),
                )
            })
            .collect();
        (value.sensor_reference.clone(), temps)
    }
}

impl From<repository::Temperature> for api::TemperatureMeasurement{
    fn from(value: repository::Temperature) -> Self {
        api::TemperatureMeasurement{
            temp_celsius: value.temp_celsius,
            measure_time: value.measure_time.and_utc(),
        }
    }
}

impl From<(String, Vec<repository::Temperature>)> for api::SensorTempMeasurements {
    fn from(value: (String, Vec<repository::Temperature>)) -> Self {
        let (sensor_ref, temps) = value;
        let temps: Vec<api::TemperatureMeasurement> = temps
            .into_iter()
            .map(|t| {
                t.into()
            })
            .collect();
        SensorTempMeasurements { sensor_reference: sensor_ref, temp_measurements: temps }
    }
}
