// api_db_conv.rs

use std::collections::HashSet;

use schili_api::api::{self, SensorSimpleMeasurements};

use crate::repository::{self, DBSimpleMeasurement};

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
            api::SensorType::BatteryVoltage => repository::SensorType::BatteryVoltage,
            api::SensorType::ChipTemperature => repository::SensorType::ChipTemperature,
            api::SensorType::Co2 => repository::SensorType::Co2,
        }
    }
}

impl ModelFrom<&repository::SensorType> for api::SensorType {
    fn model_from(value: &repository::SensorType) -> Self {
        match value {
            repository::SensorType::Temperature => api::SensorType::Temperature,
            repository::SensorType::Humidity => api::SensorType::Humidity,
            repository::SensorType::Airpressure => api::SensorType::Airpressure,
            repository::SensorType::BatteryVoltage => api::SensorType::BatteryVoltage,
            repository::SensorType::ChipTemperature => api::SensorType::ChipTemperature,
            repository::SensorType::Co2 => api::SensorType::Co2,
        }
    }
}

impl ModelFrom<&api::Sensor> for repository::Sensor {
    fn model_from(value: &api::Sensor) -> Self {
        let sensor_types: Vec<repository::SensorType> = value
            .sensor_types
            .iter()
            .map(|st| st.model_into())
            .collect();
        repository::Sensor::new(&value.reference, &value.name, sensor_types)
    }
}

impl ModelFrom<&repository::Sensor> for api::Sensor {
    fn model_from(value: &repository::Sensor) -> Self {
        let sensor_types: HashSet<api::SensorType> = value
            .sensor_types
            .iter()
            .map(|st| st.model_into())
            .collect();
        api::Sensor::new(&value.sensor_reference, &value.sensor_name, sensor_types)
    }
}

impl<T: DBSimpleMeasurement> ModelFrom<&api::SensorSimpleMeasurements> for (String, Vec<T>) {
    fn model_from(value: &api::SensorSimpleMeasurements) -> Self {
        let temps: Vec<T> = value
            .measurements
            .iter()
            .map(|t| {
                DBSimpleMeasurement::new(t.measurement.clone(), t.measure_time.naive_utc().clone())
            })
            .collect();
        (value.sensor_reference.clone(), temps)
    }
}

impl<T: DBSimpleMeasurement> ModelFrom<&api::SensorSingleSimpleMeasure> for (String, T) {
    fn model_from(value: &api::SensorSingleSimpleMeasure) -> Self {
        let measurement: T = (&value.measure).model_into();
        (value.sensor_reference.clone(), measurement)
    }
}

impl<T: DBSimpleMeasurement> ModelFrom<T> for api::SimpleMeasurement {
    fn model_from(value: T) -> Self {
        api::SimpleMeasurement {
            measurement: value.measurement().clone(),
            measure_time: value.measure_time().and_utc(),
        }
    }
}

impl<T: DBSimpleMeasurement> ModelFrom<&api::SimpleMeasurement> for T {
    fn model_from(value: &api::SimpleMeasurement) -> Self {
        T::new(value.measurement.clone(), value.measure_time.naive_utc())
    }
}

impl<T: DBSimpleMeasurement> ModelFrom<(String, Vec<T>)> for api::SensorSimpleMeasurements {
    fn model_from(value: (String, Vec<T>)) -> Self {
        let (sensor_ref, temps) = value;
        let temps: Vec<api::SimpleMeasurement> =
            temps.into_iter().map(|t| t.model_into()).collect();
        SensorSimpleMeasurements {
            sensor_reference: sensor_ref,
            measurements: temps,
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
        api::Co2Measurement {
            co2_ppm: value.co2_ppm,
            res0: value.res0,
            adc_val: value.adc_val_12bit,
            measure_time: value.measure_time.and_utc(),
        }
    }
}

impl ModelFrom<&api::Co2Measurement> for repository::Co2 {
    fn model_from(value: &api::Co2Measurement) -> Self {
        repository::Co2 {
            co2_id: -1,
            sensor_id: -1,
            co2_ppm: value.co2_ppm.clone(),
            res0: value.res0.clone(),
            adc_val_12bit: value.adc_val,
            measure_time: value.measure_time.naive_utc(),
        }
    }
}

impl ModelFrom<&api::SensorError> for (String, repository::SensorError) {
    fn model_from(value: &api::SensorError) -> Self {
        let sensor_error= (&value.error).model_into();
        (value.sensor_reference.clone(), sensor_error)
    }
}

impl ModelFrom<repository::SensorError> for api::Error{
    fn model_from(value: repository::SensorError) -> Self {
        api::Error{
            error_code: value.error_code.into(),
            error_text: value.error_text,
            error_time: value.error_time.and_utc(),
        }
    }
}

impl ModelFrom<&api::Error> for repository::SensorError{
    fn model_from(value: &api::Error) -> Self {
        repository::SensorError{
            sensor_id: -1,
            sensor_error_id: -1,
            error_code: value.error_code.get_code(),
            error_text: value.error_text.clone(),
            error_time: value.error_time.naive_utc(),
        }
    }
}


