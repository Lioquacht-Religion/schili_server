// http_server.rs

use actix_web::{
    App, HttpServer, Responder, get,
    middleware::Logger,
    post,
    web::{self, ThinData},
};
use sqlx::{Pool, Postgres};

use schili_api::api;

use crate::{database, error::ApiError, service};

pub async fn start_http_server() -> std::io::Result<()> {
    let pool = database::create_db_pool().await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(AppState {
                app_name: String::from("Schili Sensor Server"),
            }))
            .app_data(web::ThinData(pool.clone()))
            .service(web::scope("/app").route("/index.html", web::get().to(index)))
            .service(post_sensor)
            .service(post_temperature_all)
            .service(get_sensor_temperatures_all)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

struct AppState {
    app_name: String,
}

async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[post("/sensor/add")]
async fn post_sensor(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    api_sensor: web::Json<api::Sensor>,
) -> actix_web::Result<impl Responder, ApiError> {
    //TODO: db error handling, unique indexes,
    // check if sensor with reference already exists
    let db_sensor = service::add_sensor(&pool, &api_sensor).await?;

    Ok(format!(
        "Added sensor with reference='{}' and name='{}'.",
        db_sensor.sensor_reference, db_sensor.sensor_name
    ))
}

#[post("/sensor/temperature/add/all")]
async fn post_temperature_all(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    api_temp_measures: web::Json<api::SensorTempMeasurements>,
) -> actix_web::Result<impl Responder, ApiError> {
    match service::insert_temperatures_all(&pool, &api_temp_measures).await {
        Ok(()) => Ok("added sensor temperature measurements."),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[get("/sensor/temperature/{sensor_reference}")]
async fn get_sensor_temperatures_all(
    path: web::Path<(String,)>,
    ThinData(pool): web::ThinData<Pool<Postgres>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let (sensor_ref,) = &path.into_inner();
    match service::get_sensor_temperatures_all(&pool, sensor_ref.to_owned()).await {
        Ok(api_temps) => Ok(web::Json(api_temps)),
        Err(e) => Err(ApiError::from(e)),
    }
}
