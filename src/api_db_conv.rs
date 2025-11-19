// api_db_conv.rs

use std::collections::HashSet;

use schili_api::api::{self, SensorTempMeasurements};

use crate::repository;

pub trait ModelFrom<T>: Sized {
    fn model_from(value: T) -> Self;
}

pub trait ModelInto<T>: Sized {
    fn model_into(self) -> T;
}

impl<T, U: ModelFrom<T>> ModelInto<U> for T {
    fn model_into(self) -> U {
        U::model_from(self)
    }
}

impl<T> ModelFrom<T> for T {
    fn model_from(value: T) -> Self {
        value
    }
}

impl ModelFrom<&api::SensorType> for repository::SensorType {
    fn model_from(value: &api::SensorType) -> Self {
        match value {
            api::SensorType::Temperature => repository::SensorType::Temperature,
            api::SensorType::Humidity => repository::SensorType::Humidity,
            api::SensorType::Airpressure => repository::SensorType::Airpressure,
            api::SensorType::Co2 => repository::SensorType::Co2,
        }
    }
}

impl ModelFrom<&api::Sensor> for repository::Sensor {
    fn model_from(value: &api::Sensor) -> Self {
        let sensor_types: HashSet<repository::SensorType> = value
            .sensor_types
            .iter()
            .map(|st| st.model_into())
            .collect();
        repository::Sensor::new(&value.reference, &value.name, sensor_types)
    }
}

impl ModelFrom<&api::SensorTempMeasurements> for (String, Vec<repository::Temperature>) {
    fn model_from(value: &api::SensorTempMeasurements) -> Self {
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

impl ModelFrom<&api::SensorSingleTempMeasure> for (String, repository::Temperature) {
    fn model_from(value: &api::SensorSingleTempMeasure) -> Self {
        let temp: repository::Temperature = (&value.temp_measure).model_into();
        (value.sensor_reference.clone(), temp)
    }
}

impl ModelFrom<repository::Temperature> for api::TemperatureMeasurement {
    fn model_from(value: repository::Temperature) -> Self {
        api::TemperatureMeasurement {
            temp_celsius: value.temp_celsius,
            measure_time: value.measure_time.and_utc(),
        }
    }
}

impl ModelFrom<&api::TemperatureMeasurement> for repository::Temperature {
    fn model_from(value: &api::TemperatureMeasurement) -> Self {
        repository::Temperature{
            temperature_id: -1,
            sensor_id: -1,
            temp_celsius: value.temp_celsius.clone(),
            measure_time: value.measure_time.naive_utc(),
        }
    }
}

impl ModelFrom<&api::SensorSingleCo2Measure> for (String, repository::Co2) {
    fn model_from(value: &api::SensorSingleCo2Measure) -> Self {
        let co2: repository::Co2 = (&value.co2_measure).model_into();
        (value.sensor_reference.clone(), co2)
    }
}

impl ModelFrom<repository::Co2> for api::Co2Measurement {
    fn model_from(value: repository::Co2) -> Self {
        api::Co2Measurement{
            co2_ppm: value.co2_ppm,
            res0: value.res0,
            adc_val: value.adc_val_12bit,
            measure_time: value.measure_time.and_utc(),
        }
    }
}

impl ModelFrom<&api::Co2Measurement> for repository::Co2 {
    fn model_from(value: &api::Co2Measurement) -> Self {
        repository::Co2{
            co2_id: -1,
            sensor_id: -1,
            co2_ppm: value.co2_ppm.clone(),
            res0: value.res0.clone(),
            adc_val_12bit: value.adc_val,
            measure_time: value.measure_time.naive_utc(),
        }
    }
}

impl ModelFrom<(String, Vec<repository::Temperature>)> for api::SensorTempMeasurements {
    fn model_from(value: (String, Vec<repository::Temperature>)) -> Self {
        let (sensor_ref, temps) = value;
        let temps: Vec<api::TemperatureMeasurement> =
            temps.into_iter().map(|t| t.model_into()).collect();
        SensorTempMeasurements {
            sensor_reference: sensor_ref,
            temp_measurements: temps,
        }
    }
}
