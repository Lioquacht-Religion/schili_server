// mqtt_handler.rs

use std::time::Duration;

use log::info;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use schili_api::mq_topics::{chip_temperature_topic, sensor_temperature_topic};
use tokio::time;

static UUID: &str = "42";

pub async fn start_mq_client() {
    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let _handle = actix_rt::spawn(async move {
        client
            .subscribe(format!("{UUID}/hello/rumqtt"), QoS::AtMostOnce)
            .await
            .unwrap();

        client
            .subscribe(format!("{UUID}/hello"), QoS::AtMostOnce)
            .await
            .unwrap();
        client
            .subscribe(chip_temperature_topic(UUID), QoS::AtMostOnce)
            .await
            .unwrap();
        client
            .subscribe(sensor_temperature_topic(UUID), QoS::AtMostOnce)
            .await
            .unwrap();

        for i in 0..10 {
            client
                .publish(
                    format!("{UUID}/hello/rumqtt"),
                    QoS::AtLeastOnce,
                    false,
                    vec![i; i as usize],
                )
                .await
                .unwrap();
            time::sleep(Duration::from_millis(1000)).await;

            println!("publish");
            info!("publish");
        }
        time::sleep(Duration::from_secs(10)).await;
    });

    let _handle2 = actix_rt::spawn(async move {
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

                    if publish.topic.contains(&chip_temperature_topic(UUID)) {
                        let temp_bytes_arr: [u8; 4] =
                            publish.payload[0..4].try_into().expect("4 bytes length");
                        let temp: f32 = f32::from_be_bytes(temp_bytes_arr);
                        info!("chip temperature received: {:?}", temp);
                    }
                    if publish.topic.contains(&sensor_temperature_topic(UUID)) {
                        let temp_bytes_arr: [u8; 4] =
                            publish.payload[0..4].try_into().expect("4 bytes length");
                        let temp: f32 = f32::from_be_bytes(temp_bytes_arr);
                        info!("sensor temperature received: {:?}", temp);
                    }

                    info!("publish received: {:?}", publish);
                }
                Err(e) => {
                    println!("Received = {:?}", e);
                    log::error!("Error received = {:?}", e);
                }
            }
        }
    });
}
