use std::collections::HashSet;

use log::info;
use sensor_data_server::{
    database, http_server::start_http_server, mqtt_handler::start_mq_client,
    repository::start_sql_query,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = database::create_db_pool().await;

    start_sql_query(&pool)
        .await
        .expect("Executing start sql failed.");

    /*
    let mut sensor = Sensor {
        id: 0,
        name: "test_sensor1".into(),
        reference: "100000".into(),
        sensor_types: HashSet::new(),
    };

    repository::insert_sensor_with_sensor_types(&pool, &mut sensor)
        .await
        .expect("Insert sensor enitty failed.");
    */

    let sensor = schili_api::api::Sensor {
        name: "test_sensor1".into(),
        reference: "100000".into(),
        sensor_types: HashSet::new(),
    };

    let sensor_json = serde_json::to_string(&sensor).unwrap();
    info!("sensor json: {}", sensor_json);

    start_mq_client().await;

    start_http_server().await
}
