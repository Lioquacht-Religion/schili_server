use sensor_data_server::{
    config, email, http_server::start_http_server, mqtt_handler::start_mq_client,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let config = config::get_config().await;

    let mut dispatch_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                    "[{} {} {}] {}",
                    chrono::Local::now(),
                    record.level(),
                    record.target(),
                    message
            ));
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());
    if let Some(log_file) = &config.logging.file{
        dispatch_logger = dispatch_logger
            .chain(fern::log_file(log_file)?)
    }
    if let Err(e) = dispatch_logger
        .apply(){
            panic!("Logger configuration error: {e}")
    }

    email::send_server_started_email(&config.email);

    start_mq_client(config).await;

    start_http_server().await
}
