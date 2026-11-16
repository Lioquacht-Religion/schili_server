// mqtt_handler.rs

use std::{str::FromStr, time::Duration};

use actix_web::web::Buf;
use anyhow::anyhow;
use log::{error, info};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Publish, QoS, StateError};
use schili_api::{
    api::SensorType, mq_topics::{
        Dataformat, TOPICS
    },
};
use sqlx::{Pool, Postgres};
use tokio::io;

use crate::{
    SensorCache, config::Config, database, service, topic_router::{BoxFuture, TopicRouteParseError, TopicRouter, TopicValue, macros::into_async3},
};

pub async fn start_mq_client(app_config: &Config, sensor_cache: SensorCache) {
    let mut mqttoptions = MqttOptions::new(
        &app_config.mqtt.broker_id,
        &app_config.mqtt.host,
        app_config.mqtt.port,
    );
    mqttoptions.set_credentials(&app_config.mqtt.username, &app_config.mqtt.passw);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let pool = database::create_db_pool().await;

    let _handle = actix_rt::spawn(async move {
        loop {
            let _ = match subscribe_to_topics(&client).await {
                Ok(router) => {
                    handle_mq_events(&router, &mut eventloop, &pool, sensor_cache.clone()).await;
                }
                Err(e) => {
                    error!("Received while parsing topic routes. Error: {:?}", e);
                }
            };
        }
    });
}

async fn handle_simple_measurements<'o, 'v>(
    topic_values: &'v [TopicValue<'o>],
    publish: &'o Publish,
    pool: &'o Pool<Postgres>,
    sensor_cache: &'o SensorCache,
) -> anyhow::Result<()> {
    if let [
        TopicValue::IntVar(_uuid),
        TopicValue::EnumVar(dataformat),
        TopicValue::EnumVar(measurement),
        TopicValue::EnumVar(_source),
    ] = topic_values
    {
        match SensorType::from_str(*measurement) {
            Ok(sensor_type) => {
                let measurement = match Dataformat::from_str(*dataformat) {
                    Ok(Dataformat::Json) => extract_sensor_simple_measurement_from_json(publish)?,
                    Ok(Dataformat::MsgPack) => {
                        extract_sensor_simple_measurement_from_msgpack(publish)?
                    }
                    Err(()) => {
                        return Err(anyhow!(
                            "Received unknown dataformat in topic. Received dataformat: {dataformat}"
                        ));
                    }
                };
                let sensor = 
                    sensor_cache.get_sensor_by_ref(pool, &measurement.sensor_reference).await?;
                let warn_limits = sensor.sensor_w_warn_limits.sensor_type_warn_limits.get(&sensor_type);
                match sensor_type {
                    SensorType::Co2 => {
                        return Err(anyhow!(
                            "Measurement type Co2 is not supported by this handler."
                        ));
                    }
                    _sensor_type => 
                        service::add_measurement_check_warn_limits(
                            pool, sensor_type, &sensor, warn_limits, 
                            &measurement.measure).await,
                }
            }
            Err(()) => Err(anyhow!(
                "Received unknown measurement type in topic. Received type: {measurement}"
            )),
        }
    } else {
        Err(anyhow!("Topic does not match handler pattern."))
    }
}

async fn handle_co2_measurement<'o, 'v>(
    topic_values: &'v [TopicValue<'o>],
    publish: &'o Publish,
    pool: &'o Pool<Postgres>,
    sensor_cache: &'o SensorCache,
) -> anyhow::Result<()> {
    if let [
        TopicValue::IntVar(_uuid),
        TopicValue::EnumVar(dataformat),
        TopicValue::EnumVar(measurement),
        TopicValue::EnumVar(_source),
    ] = topic_values
    {
        match SensorType::from_str(*measurement) {
            Ok(sensor_type) => {
                let measurement = match Dataformat::from_str(*dataformat) {
                    Ok(Dataformat::Json) => extract_sensor_co2_from_json(publish)?,
                    Ok(Dataformat::MsgPack) => extract_sensor_co2_from_msgpack(publish)?,
                    Err(()) => {
                        return Err(anyhow!(
                            "Received unknown dataformat in topic. Received dataformat: {dataformat}"
                        ));
                    }
                };
                if let SensorType::Co2 = sensor_type {
                    service::insert_co2(pool, sensor_cache.clone(), &measurement).await
                } else {
                    Err(anyhow!(
                        "Measurement type Co2 is not supported by this handler."
                    ))
                }
            }
            Err(()) => Err(anyhow!(
                "Received unknown measurement type in topic. Received type: {measurement}"
            )),
        }
    } else {
        Err(anyhow!("Topic does not match handler pattern."))
    }
}

async fn handle_measurements_bundle<'o, 'v>(
    topic_values: &'v [TopicValue<'o>],
    publish: &'o Publish,
    pool: &'o Pool<Postgres>,
    sensor_cache: &'o SensorCache,
) -> anyhow::Result<()> {
    if let [TopicValue::IntVar(_uuid), TopicValue::EnumVar(dataformat)] = topic_values {
        let mut measurements = match Dataformat::from_str(*dataformat) {
            Ok(Dataformat::Json) => extract_sensor_measurement_bundle_from_json(publish)?,
            Ok(Dataformat::MsgPack) => extract_sensor_measurement_bundle_from_msgpack(publish)?,
            Err(()) => {
                return Err(anyhow!(
                    "Received unknown dataformat in topic. Received dataformat: {dataformat}"
                ));
            }
        };
        service::insert_bundled_measurements(pool, sensor_cache.clone(), &mut measurements)
            .await
            .map_err(|e| {
                anyhow!(
                    "Received errors while trying to insert measurement bundle. Error: {:?}",
                    e
                )
            })
    } else {
        Err(anyhow!("Topic does not match handler pattern."))
    }
}

async fn handle_errors<'o, 'v>(
    topic_values: &'v [TopicValue<'o>],
    publish: &'o Publish,
    pool: &'o Pool<Postgres>,
    sensor_cache: &'o SensorCache,
) -> anyhow::Result<()> {
    if let [TopicValue::IntVar(_uuid), TopicValue::EnumVar(dataformat)] = topic_values {
        let sensor_error = match Dataformat::from_str(*dataformat) {
            Ok(Dataformat::Json) => extract_sensor_error_from_json(publish)?,
            Ok(Dataformat::MsgPack) => extract_sensor_error_from_msgpack(publish)?,
            Err(()) => {
                return Err(anyhow!(
                    "Received unknown dataformat in topic. Received dataformat: {dataformat}"
                ));
            }
        };
        service::insert_sensor_error(pool, sensor_cache.clone(), &sensor_error)
            .await
            .map_err(|e| {
                anyhow!(
                    "Received errors while trying to insert measurement bundle. Error: {:?}",
                    e
                )
            })
    } else {
        Err(anyhow!("Topic does not match handler pattern."))
    }
}

type Func = for<'o, 'v> fn(
    &'v [TopicValue<'o>],
    &'o Publish,
    &'o Pool<Postgres>,
    &'o SensorCache,
) -> BoxFuture<'v, anyhow::Result<()>>;

async fn subscribe_to_topics(
    client: &AsyncClient,
) -> Result<TopicRouter<Func>, TopicRouteParseError> {
    let mut router: TopicRouter<Func> = TopicRouter::new();

    router.add_route(
        "{integer}/[json|msgpack]/[temperature|humidity|airpressure|lightintensity]/[sensor|chip]",
        into_async3!(handle_simple_measurements),
    )?;
    router.add_route(
        "{integer}/[json|msgpack]/co2/sensor",
        into_async3!(handle_co2_measurement),
    )?;
    router.add_route(
        "{integer}/[json|msgpack]/measurement/bundle/sensor",
        into_async3!(handle_measurements_bundle),
    )?;
    router.add_route(
        "{integer}/[json|msgpack]/error/sensor",
        into_async3!(handle_errors),
    )?;

    let topics = [
        &TOPICS.measurement_bundle,
        &TOPICS.measurement_bundle_msgpack,
        &TOPICS.chip_temp,
        &TOPICS.chip_temp_msgpack,
        &TOPICS.temp,
        &TOPICS.temp_msgpack,
        &TOPICS.humidity,
        &TOPICS.humidity_msgpack,
        &TOPICS.air_pressure,
        &TOPICS.air_pressure_msgpack,
        &TOPICS.light_intensity,
        &TOPICS.light_intensity_msgpack,
        &TOPICS.battery_voltage,
        &TOPICS.battery_voltage_msgpack,
        &TOPICS.co2,
        &TOPICS.co2_msgpack,
        &TOPICS.error,
        &TOPICS.error_msgpack,
    ]
        .into_iter()
        .map(|t| rumqttc::SubscribeFilter { path: t.to_owned(), qos: QoS::AtLeastOnce });
    if let Err(e) = client.subscribe_many(topics).await {
        error!("Error while subscribing: {e}");
    }
    Ok(router)
}

struct DisconnectOccured;

async fn handle_mq_events(
    router: &TopicRouter<Func>,
    eventloop: &mut EventLoop,
    db_pool: &Pool<Postgres>,
    sensor_cache: SensorCache,
) -> DisconnectOccured {
    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                if let Err(es) = handle_publish(router, db_pool, &publish, sensor_cache.clone()).await {
                        error!("An error occured while trying to process published messages for topic='{}': error: {es}", publish.topic);
                };
            }
            Ok(_event) => {
                continue;
            }
            Err(e) => {
                if let rumqttc::ConnectionError::MqttState(StateError::Io(e)) = e {
                    if let io::ErrorKind::ConnectionAborted = e.kind() {
                        error!("Error received connection aborted = {:?}", e);
                        return DisconnectOccured;
                    }
                }
                else{
                    error!("Error received = {:?}", e);
                }
            }
        }
    }
}

async fn handle_publish<'tp, 'pu, 'po>(
    router: &'tp TopicRouter<Func>,
    pool: &'po Pool<Postgres>,
    publish: &'pu Publish,
    sensor_cache: SensorCache,
) -> anyhow::Result<()> {
    info!("Publish received for topic: {}", &publish.topic);
    router.exec_handler_for_route(&(publish, pool, &sensor_cache), &publish.topic)
        .await
        .map_err(|e| anyhow!("Received error from handler execution. Error: {e:?}"))
}

fn extract_sensor_simple_measurement_from_json(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleSimpleMeasure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_simple_measurement_from_msgpack(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleSimpleMeasure> {
    Ok(rmp_serde::from_read(publish.payload.clone().reader())?)
}

fn extract_sensor_measurement_bundle_from_json(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorTypedSimpleMeasurements> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    info!("measure bundle: {}", &json_str);
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_measurement_bundle_from_msgpack(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorTypedSimpleMeasurements> {
    let bundle = rmp_serde::from_read(publish.payload.clone().reader())?;
    info!("measure bundle: {:?}", &bundle);
    Ok(bundle)
}

fn extract_sensor_co2_from_json(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleCo2Measure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_co2_from_msgpack(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleCo2Measure> {
    Ok(rmp_serde::from_read(publish.payload.clone().reader())?)
}

fn extract_sensor_error_from_json(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorError> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_error_from_msgpack(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorError> {
    Ok(rmp_serde::from_read(publish.payload.clone().reader())?)
}
