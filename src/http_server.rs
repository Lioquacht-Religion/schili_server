// http_server.rs

use actix_web::{get, middleware::Logger, post, web::{self, ThinData}, App, HttpResponse, HttpServer, Responder, ResponseError};
use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;

use crate::{api, database, repository};

pub async fn start_http_server() -> std::io::Result<()> {
    let counter = web::Data::new(AppStatWithCounter {
        counter: Mutex::new(0),
    });

    let pool = database::create_db_pool().await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(AppState {
                app_name: String::from("Schili Sensor Server"),
            }))
            .app_data(counter.clone())
            .app_data(web::ThinData(pool.clone()))
            .service(count)
            .service(web::scope("/app").route("/index.html", web::get().to(index)))
            .service(sensor_add)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

struct AppStatWithCounter {
    counter: Mutex<i32>,
}

struct AppState {
    app_name: String,
}

async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[get("/count")]
async fn count(data: web::Data<AppStatWithCounter>) -> impl Responder {
    let mut counter = data.counter.lock().await;
    *counter += 1;
    HttpResponse::Ok().body(format!("Request number: {counter}"))
}

#[post("/sensor/add")]
async fn sensor_add(ThinData(pool): web::ThinData<Pool<Postgres>>, api_sensor: web::Json<api::Sensor>) -> actix_web::Result<impl Responder>{
            //TODO: db error handling, unique indexes, 
            // check if sensor with referenc already exists
            let mut db_sensor : repository::Sensor = (&*api_sensor).into();
            if let Err(e) = repository::insert_sensor_with_sensor_types(&pool, &mut db_sensor).await{
                return Err(e.into());
            }

            Ok(format!("added sensor with name: {}", db_sensor.name))
}

#[post("/temperature/add")]
async fn temperature_add(ThinData(pool): web::ThinData<Pool<Postgres>>, api_sensor: web::Json<api::Sensor>) -> actix_web::Result<impl Responder>{
            //TODO: db error handling, unique indexes, 
            // check if sensor with referenc already exists
            let mut db_sensor : repository::Sensor = (&*api_sensor).into();
            if let Err(e) = repository::insert_sensor_with_sensor_types(&pool, &mut db_sensor).await{
                return Err(e.into());
            }

            Ok(format!("added sensor with name: {}", db_sensor.name))
}
