// http_server.rs

use std::str::FromStr;

use actix_cors::Cors;
use actix_web::{
    App, HttpServer, Responder, get, http,
    middleware::Logger,
    post,
    web::{self, ThinData},
};
use anyhow::anyhow;
use chrono::Local;
use log::error;
use sqlx::{Pool, Postgres};

use schili_api::api::{
    self, GetSensorSimpleMeasuresIntervalsRange, GetSensorSimpleMeasuresRange, SensorType,
};

use crate::{
    SensorCache, api_db_conv::ModelInto, config, database, error::{ApiError, DateRangeError}, repository::{self}, service::{self},
};

pub async fn start_http_server(sensor_cache: SensorCache) -> std::io::Result<()> {
    let pool = database::create_db_pool().await;
    let config = config::get_config().await;

    HttpServer::new(move || {
        let mut cors = Cors::default();
        for cors_host in config.http_service.cors_hosts.iter() {
            cors = cors.allowed_origin(cors_host);
        }
        let cors = cors
            .allowed_methods(vec!["GET", "POST", "UPDATE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                app_name: String::from("Schili Sensor Server"),
            }))
            .app_data(web::ThinData(pool.clone()))
            .app_data(web::ThinData(sensor_cache.clone()))
            .service(web::scope("/app").route("/index.html", web::get().to(index)))
            .service(post_sensor)
            .service(get_sensor)
            .service(get_all_sensors)
            .service(get_all_sensors_filtered)
            .service(post_measurement_all)
            .service(get_sensor_measurements_range)
            .service(get_sensor_avg_simple_measurement_interval_in_range)
            .service(get_current_datetime)
    })
    .bind((config.http_service.host.as_str(), config.http_service.port))?
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

#[get("/datetime/current/local")]
async fn get_current_datetime() -> actix_web::Result<impl Responder, ApiError> {
    let current_datetime = Local::now().naive_local();
    Ok(web::Json(current_datetime))
}

#[post("/sensor/add")]
async fn post_sensor(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    api_sensor: web::Json<api::SensorWithWarnLimits>,
) -> actix_web::Result<impl Responder, ApiError> {
    //TODO: db error handling, unique indexes,
    // check if sensor with reference already exists
    let db_sensor = service::add_sensor(&pool, &api_sensor).await?;

    Ok(format!(
        "Added sensor with reference='{}' and name='{}'.",
        db_sensor.sensor_reference, db_sensor.sensor_name
    ))
}

#[get("/sensor/reference/{sensor_ref}")]
async fn get_sensor(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder, ApiError> {
    let sensor_ref = path.into_inner();
    let sensor = service::get_sensor(&pool, &sensor_ref).await?;
    Ok(web::Json(sensor))
}

#[get("/sensor/all")]
async fn get_all_sensors(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let sensors = service::get_all_sensors(&pool).await?;
    Ok(web::Json(sensors))
}

#[actix_web::post("/sensor/filtered/{sensor_name_part}")]
async fn get_all_sensors_filtered(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    path: web::Path<String>,
    api_sensor_types: web::Json<Vec<api::SensorType>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let sensor_name_filter = format!("%{}%", path.into_inner());
    let sensor =
        service::get_all_sensors_filtered(&pool, &sensor_name_filter, &api_sensor_types).await?;
    Ok(web::Json(sensor))
}

#[post("/sensor/{sensor_ref}/{measurement_kind}/add/all")]
async fn post_measurement_all(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    ThinData(sensor_cache): web::ThinData<SensorCache>,
    path: web::Path<(String, String)>,
    api_temp_measures: web::Json<Vec<api::SimpleMeasurement>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let (sensor_ref, measurement_kind) = path.into_inner();
    let sensor = sensor_cache.get_sensor_by_ref(&pool, &sensor_ref).await?;
    let measure_type: SensorType = SensorType::from_str(&measurement_kind)
        .map_err(|()| anyhow!("Measurement kind not allowed."))?;
    //let measure_type : SensorTypeDB = measure_type.into();
    let mut measurements = api_temp_measures.into_inner()
        .into_iter()
        .map(|m| (measure_type, m).model_into())
        .collect();
    repository::insert_sensor_measurements(
        &pool, sensor.sensor_id, &mut measurements
    ).await
        .map_err(
            |e| anyhow!("Failed to insert measurements. Error: {e}")
        )?;
    Ok("Added sensor temperature measurements.")
}

#[get(
    "/sensor/measurement/range/{measurement_kind}/{sensor_reference}/{start_datetime}/{end_datetime}"
)]
async fn get_sensor_measurements_range(
    path: web::Path<(String, String, i64, i64)>,
    ThinData(pool): web::ThinData<Pool<Postgres>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let (measurement_kind, sensor_ref, start, end) = path.into_inner();
    let measurement_kind = SensorType::from_str(&measurement_kind).map_err(|_| {
        anyhow!(
            "Measurement kind does not exist: {}",
            measurement_kind.as_str()
        )
    })?;
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

    match service::get_sensor_measurements_in_range(&pool, &temp_range, measurement_kind).await {
        Ok(api_temps) => Ok(web::Json(api_temps)),
        Err(e) => Err(ApiError::from(e)),
    }
}

#[get(
    "/sensor/measurement/avg/interval/range/{measurement_kind}/{sensor_reference}/{start_datetime}/{end_datetime}/{interval}"
)]
async fn get_sensor_avg_simple_measurement_interval_in_range(
    path: web::Path<(String, String, i64, i64, i64)>,
    ThinData(pool): web::ThinData<Pool<Postgres>>,
) -> actix_web::Result<impl Responder, ApiError> {
    let (measurement_kind, sensor_ref, start, end, interval) = path.into_inner();
    let measurement_kind = SensorType::from_str(&measurement_kind).map_err(|_| {
        anyhow!(
            "Measurement kind does not exist: {}",
            measurement_kind.as_str()
        )
    })?;
    let start_datetime = chrono::DateTime::from_timestamp(start, 0);
    let end_datetime = chrono::DateTime::from_timestamp(end, 0);
    let interval = chrono::TimeDelta::milliseconds(interval);
    let temp_range =
        if let (Some(start_datetime), Some(end_datetime)) = (start_datetime, end_datetime) {
            GetSensorSimpleMeasuresIntervalsRange {
                sensor_reference: sensor_ref,
                start_datetime,
                end_datetime,
                interval,
            }
        } else {
            return Err(ApiError::from(anyhow!(DateRangeError::from((
                start_datetime,
                end_datetime
            )))));
        };

    match service::get_sensor_avg_measurements_by_intervals_in_range(
        &pool,
        &temp_range,
        measurement_kind,
    )
    .await
    {
        Ok(api_temps) => Ok(web::Json(api_temps)),
        Err(e) => {
            error!(
                "Error while trying to search for intervals in timerange: {}",
                e
            );
            Err(ApiError::from(e))
        }
    }
}
