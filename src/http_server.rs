// http_server.rs

use actix_web::{
    App, HttpServer, Responder, get,
    middleware::Logger,
    post,
    web::{self, ThinData},
};
use sqlx::{Pool, Postgres};

use crate::{api, database, repository};

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
) -> actix_web::Result<impl Responder> {
    //TODO: db error handling, unique indexes,
    // check if sensor with referenc already exists
    let mut db_sensor: repository::Sensor = (&*api_sensor).into();
    if let Err(e) = repository::insert_sensor_with_sensor_types(&pool, &mut db_sensor).await {
        return Err(e.into());
    }

    Ok(format!("added sensor with name: {}", db_sensor.sensor_name))
}

#[post("/sensor/temperature/add/all")]
async fn post_temperature_all(
    ThinData(pool): web::ThinData<Pool<Postgres>>,
    api_temp_measures: web::Json<api::SensorTempMeasurements>,
) -> actix_web::Result<impl Responder> {
    let (sensor_ref, mut db_temps): (String, Vec<repository::Temperature>) =
        (&*api_temp_measures).into();
    match repository::find_sensor_by_ref(&pool, &sensor_ref).await {
        Ok(sensor) => {
            if let Err(e) =
                repository::insert_sensor_temperature_measures(&pool, sensor.sensor_id, &mut db_temps)
                    .await
            {
                return Err(e.into());
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok("added sensor temperature measurements.")
}

#[get("/sensor/temperature/{sensor_reference}")]
async fn get_sensor_temperatures_all(
    path: web::Path<(String,)>,
    ThinData(pool): web::ThinData<Pool<Postgres>>,
) -> actix_web::Result<impl Responder> {
    let (sensor_ref,) = &path.into_inner();
    match repository::find_sensor_by_ref(&pool, &sensor_ref).await {
        Ok(sensor) => {
                let temps = repository::find_sensor_temperature_measures(&pool, sensor.sensor_id).await?;
                let sensor_temps = (sensor_ref.to_owned(), temps); 
                let api_temps : api::SensorTempMeasurements = sensor_temps.into();
                Ok(web::Json(api_temps))
        }
        Err(e) => Err(e.into()),
    }
}
