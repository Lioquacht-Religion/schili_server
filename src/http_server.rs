// http_server.rs

use actix_web::{
    App, HttpServer, Responder, get,
    middleware::Logger,
    post,
    web::{self, ThinData},
};
use anyhow::anyhow;
use sqlx::{Pool, Postgres};

use schili_api::api::{self, GetSensorSimpleMeasuresRange};

use crate::{
    database,
    error::{ApiError, DateRangeError},
    service,
};

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
            .service(get_sensor_temperatures_range)
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
    api_temp_measures: web::Json<api::SensorSimpleMeasurements>,
) -> actix_web::Result<impl Responder, ApiError> {
    match service::insert_temperatures_all(&pool, &api_temp_measures).await {
        Ok(()) => Ok("Added sensor temperature measurements."),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[post("/sensor/humidity/add/all")]
async fn post_humidity_all(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    api_temp_measures: web::Json<api::SensorSimpleMeasurements>,
) -> actix_web::Result<impl Responder, ApiError> {
    match service::insert_temperatures_all(&pool, &api_temp_measures).await {
        Ok(()) => Ok("Added sensor humidity measurements."),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[get("/sensor/temperature/{sensor_reference}")]
#[deprecated]
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

#[get("/sensor/temperature/range/{sensor_reference}/{start_datetime}/{end_datetime}")]
async fn get_sensor_temperatures_range(
    path: web::Path<(String, i64, i64)>,
    ThinData(pool): web::ThinData<Pool<Postgres>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let (sensor_ref, start, end) = path.into_inner();
    let start_datetime = chrono::DateTime::from_timestamp(start, 0);
    let end_datetime = chrono::DateTime::from_timestamp(end, 0);
    let temp_range =
        if let (Some(start_datetime), Some(end_datetime)) = (start_datetime, end_datetime) {
            GetSensorSimpleMeasuresRange {
                sensor_reference: sensor_ref,
                start_datetime,
                end_datetime,
            }
        } else {
            return Err(ApiError::from(anyhow!(DateRangeError::from((
                start_datetime,
                end_datetime
            )))));
        };

    match service::get_sensor_temperatures_in_range(&pool, &temp_range).await {
        Ok(api_temps) => Ok(web::Json(api_temps)),
        Err(e) => Err(ApiError::from(e)),
    }
}
