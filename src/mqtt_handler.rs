// mqtt_handler.rs

use std::time::Duration;

use chrono::Utc;
use log::{error, info};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Publish, QoS};
use schili_api::mq_topics::{chip_temperature_topic, sensor_airpressure_topic, sensor_co2_topic, sensor_humidity_topic, sensor_temperature_topic, TOPICS};
use sqlx::{Pool, Postgres};

use crate::{config::Config, database, service};

static UUID: &str = "42";

pub async fn start_mq_client(app_config: &Config) {
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
        subscribe_to_topics(&client).await;
        handle_mq_events(&mut eventloop, &pool).await;
    });
}

async fn subscribe_to_topics(client: &AsyncClient) {
    client
        .subscribe(chip_temperature_topic(UUID), QoS::AtMostOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_temperature_topic(UUID), QoS::AtMostOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_co2_topic(UUID), QoS::AtMostOnce)
        .await
        .unwrap();
}

async fn handle_mq_events(eventloop: &mut EventLoop, db_pool: &Pool<Postgres>) {
    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(notification) => {
                info!("Received = {:?}", notification);
                let publish = if let Event::Incoming(Packet::Publish(publish)) = notification {
                    publish
                } else {
                    continue;
                };

                if let Err(e) = handle_publish(db_pool, &publish).await{
                    error!("An error occured while trying to process published messages: error: {}", e);
                };
            }
            Err(e) => {
                println!("Received = {:?}", e);
                error!("Error received = {:?}", e);
            }
        }
    }
}

async fn handle_publish(pool: &Pool<Postgres>, publish: &Publish) -> anyhow::Result<()>{
    if publish.topic.contains(&TOPICS.chip_temp) {
        let chip_temp = extract_sensor_simple_measurement(publish)?;
        service::insert_chip_temperature(pool, &chip_temp).await?;
    }
    if publish.topic.contains(&TOPICS.temp) {
        let mut sens_temps = extract_sensor_simple_measurement(&publish)?;
        sens_temps.measure.measure_time = Utc::now();
        service::insert_temperature(pool, &sens_temps).await?;
    }
    if publish.topic.contains(&TOPICS.humidity) {
        let mut sens_hums = extract_sensor_simple_measurement(&publish)?;
        sens_hums.measure.measure_time = Utc::now();
        service::insert_humidity(pool, &sens_hums).await?;
    }
    if publish.topic.contains(&TOPICS.air_pressure) {
        let mut sens_hums = extract_sensor_simple_measurement(&publish)?;
        sens_hums.measure.measure_time = Utc::now();
        service::insert_humidity(pool, &sens_hums).await?;
    }
    if publish.topic.contains(&TOPICS.co2) {
        let mut sens_co2 = extract_sensor_co2(&publish)?;
        sens_co2.co2_measure.measure_time = Utc::now();
        if let Err(e) = service::insert_co2(pool, &sens_co2).await {
            error!("Could not insert co2 from mq publish. error: {}", e);
        }

        info!(
            "sensor temps: {}",
            serde_json::to_string(&sens_co2).unwrap()
        );
    }
    Ok(())
}

fn extract_sensor_simple_measurement(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleSimpleMeasure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec()).unwrap();
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_co2(publish: &Publish) -> anyhow::Result<schili_api::api::SensorSingleCo2Measure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_temperature(publish: &Publish) -> f32 {
    let temp_bytes_arr: [u8; 4] = publish.payload[0..4].try_into().expect("4 bytes length");
    let temp: f32 = f32::from_be_bytes(temp_bytes_arr);
    temp
}
