// api_db_conv.rs

use std::collections::HashSet;

use crate::{api, repository};

impl From<&api::SensorType> for repository::SensorType{
    fn from(value: &api::SensorType) -> Self {
        match value{
            api::SensorType::Temperature => repository::SensorType::Temperature,
            api::SensorType::Humidity => repository::SensorType::Humidity,
            api::SensorType::Airpressure => repository::SensorType::Airpressure,
            api::SensorType::Co2 => repository::SensorType::Co2,
        }
    }
}

impl From<&api::Sensor> for repository::Sensor{
    fn from(value: &api::Sensor) -> Self {
        let sensor_types : HashSet<repository::SensorType> = value.sensor_types
            .iter().map(|st| st.into())
            .collect();
       repository::Sensor::new(&value.reference, &value.name, sensor_types) 
    }
}
