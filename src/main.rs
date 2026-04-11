use sensor_data_server::{
    config, email, http_server::start_http_server, mqtt_handler::start_mq_client,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = config::get_config().await;

    email::send_server_started_email(&config.email);

    start_mq_client(config).await;

    start_http_server().await
}
