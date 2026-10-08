// mqtt_handler.rs

use std::time::Duration;

use log::{error, info};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Publish, QoS, StateError};
use schili_api::mq_topics::{
    TOPICS, sensor_co2_topic, sensor_error_topic, sensor_measurements_bundle_topic
};
use sqlx::{Pool, Postgres};
use tokio::io;

use crate::{config::Config, database, service, topic_router::{BoxFuture, TopicRouter, TopicValue, macros::into_async2}};

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
        loop{
            let router = subscribe_to_topics(&client).await;
            handle_mq_events(&router, &mut eventloop, &pool).await;
        }
    });
}

async fn handle_chip_temp<'a>(input: &(&Publish, &Pool<Postgres>), _topic_values: &[TopicValue<'a>]) -> anyhow::Result<()>{
        let chip_temp = extract_sensor_simple_measurement(input.0)?;
        service::insert_chip_temperature(input.1, &chip_temp).await
}

async fn handle_temp<'o, 'v>(_topic_values: &'v [TopicValue<'o>], input: &'o Publish, input2: &'o Pool<Postgres>) -> anyhow::Result<()>{
        let sens_temps = extract_sensor_simple_measurement(input)?;
        service::insert_temperature_w_sensor(input2, &sens_temps).await
}

type Func = for<'o, 'v> fn(&'v[TopicValue<'o>], &'o Publish, input2: &'o Pool<Postgres>) -> BoxFuture<'v, anyhow::Result<()>>;

async fn subscribe_to_topics(client: &AsyncClient) -> TopicRouter<Func>{
    //let mut errors: Vec<anyhow::Error> = Vec::new();
    let mut router: TopicRouter<Func>
                = TopicRouter::new();

    /*
    subscribe_to_topic(client, &mut router, &TOPICS.chip_temp,
        into_async!(handle_chip_temp));
    subscribe_to_topic(client, &mut router, &TOPICS.temp,
            into_async!(handle_temp));
    */

    router.add_route(
        "{integer}/[json|msgpack]/[temperature|humidity|airpressure|lightintensity]/[sensor|chip]",
        into_async2!(handle_temp)
    ).unwrap();
    router.add_route(
        "{integer}/[json|msgpack]/co2/sensor",
        into_async2!(handle_temp)
    ).unwrap();

    /*
    subscribe_to_topic(
        client, &mut router, &TOPICS.humidity,
        |publish, pool| Box::new(async{
            let sens_hums = extract_sensor_simple_measurement(publish)?;
            service::insert_humidity(pool, &sens_hums).await
        }));
    subscribe_to_topic(client, &mut router, &TOPICS.air_pressure,
        |publish, pool| Box::new(async{
            let sens_hums = extract_sensor_simple_measurement(publish)?;
            service::insert_airpressure(pool, &sens_hums).await
        }));
    subscribe_to_topic(client, &mut router, &TOPICS.light_intensity,
        |publish, pool| Box::new(async{
            let sens_hums = extract_sensor_simple_measurement(publish)?;
            service::insert_airpressure(pool, &sens_hums).await
        }));
    subscribe_to_topic(client, &mut router, &TOPICS.battery_voltage,
        |publish, pool| Box::new(async{
            let sens_battv = extract_sensor_simple_measurement(publish)?;
            service::insert_battery_voltage(pool, &sens_battv).await
        }));

    subscribe_to_topic(client, &mut router, &TOPICS.co2,
        |publish, pool| Box::new(async{
            let sens_co2 = extract_sensor_co2(publish)?;
            service::insert_co2(pool, &sens_co2).await
        }));
    subscribe_to_topic(client, &mut router, &TOPICS.measurement_bundle,
        
        //TODO
        |publish, pool| Box::new(async{
        let mut sensor = extract_sensor_measurement_bundle(publish).unwrap();
        service::insert_bundled_measurements(pool, &mut sensor).await.unwrap();
        Ok(())
    }));
    subscribe_to_topic(client, &mut router, &TOPICS.error,
        |publish, pool| Box::new(async{
        let sensor_error= extract_sensor_error(publish)?;
        service::insert_sensor_error(pool, &sensor_error).await
        }));
    */

    let topics = [
        &TOPICS.measurement_bundle, &TOPICS.chip_temp, &TOPICS.temp,
        &TOPICS.humidity, &TOPICS.air_pressure, &TOPICS.light_intensity,
        &TOPICS.battery_voltage,
        &TOPICS.co2, &TOPICS.error
    ];
    for topic in topics{
        if let Err(e) = client
            .subscribe(topic, QoS::AtLeastOnce)
            .await{
                error!("Error while subscribing: {e}");
        }
    }
    router
}

struct DisconnectOccured;

async fn handle_publish2<'pu1: 'pu2, 'pu2, 'po>(
    router: &TopicRouter<Func>,
    publish: &'pu1 Publish, db_pool: &'po Pool<Postgres>)
{
    if let Err(es) = handle_publish(
        router, db_pool, &publish
    ).await {
        for e in es{
            error!(
                "An error occured while trying to process published messages: error: {e}"
            );
        }
    };
}

async fn handle_mq_events(
    router: &TopicRouter<Func>,
    eventloop: &mut EventLoop, db_pool: &Pool<Postgres>) -> DisconnectOccured
{
    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                handle_publish2(router, &publish, db_pool);
            }
            Ok(_event) => {
                continue;
            }
            Err(e) => {
                error!("Error received = {:?}", e);
                if let rumqttc::ConnectionError::MqttState(StateError::Io(e)) = e {
                    if let io::ErrorKind::ConnectionAborted = e.kind(){
                        return DisconnectOccured;
                    }
                }
            }
        }
    }
}

async fn handle_publish<'tp, 'pu, 'po>(
    router: &'tp TopicRouter<Func>,
    pool: &'po Pool<Postgres>, publish: &'pu Publish
) -> anyhow::Result<(), Vec<anyhow::Error>> {
    info!("Publish received for topic: {}", &publish.topic);
    let mut errors: Vec<anyhow::Error> = Vec::new();

    router.exec_handler_for_route(&(publish, pool), &publish.topic);

    if errors.is_empty(){
        Ok(())
    }
    else{
        Err(errors)
    }
}

fn extract_sensor_simple_measurement(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleSimpleMeasure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_measurement_bundle(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorTypedSimpleMeasurements> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    info!("measure bundle: {}", &json_str);
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_co2(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleCo2Measure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_error(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorError> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}
