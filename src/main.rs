use std::sync::Mutex;

use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};

use log::{error, info};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::time;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let _mq_client_task_handle = start_mq_client().await;
    //let _ = mq_client_task_handle.await.expect("Something failed with the mq client initialization!");

    start_http_server().await
}

async fn start_sql_query() -> std::result::Result<(), Box<dyn std::error::Error>>{
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set")).await?;
        //.connect("postgres://postgres:password@localhost/test").await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    assert_eq!(row.0, 150);

    Ok(())
}

async fn start_mq_client() { 
    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let _handle = actix_rt::spawn(async move {
        client.subscribe("hello/rumqtt", QoS::AtMostOnce).await.unwrap();
        for i in 0..10 {
            client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
            time::sleep(Duration::from_millis(1000)).await;

            println!("publish");
            info!("publish");
        }
        time::sleep(Duration::from_secs(10)).await;
    });

    let _handle2 = actix_rt::spawn(async move {
        loop{
            let event = eventloop.poll().await;
            match event {
                Ok(notification) => {
                    info!("Received = {:?}", notification);
                    let publish = if let Event::Incoming(Packet::Publish(publish)) = notification{
                       publish 
                    }
                    else {
                        continue;
                    };

                    info!("publish received: {:?}", publish);
                }
                Err(e) => {
                    println!("Received = {:?}", e);
                    error!("Error received = {:?}", e);
                }
            }
        }
    });
}

async fn start_http_server() -> std::io::Result<()>{
    let counter = web::Data::new(AppStatWithCounter{
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .app_data(counter.clone()) 
            .service(count)
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .service(web::scope("/app")
                .route("/index.html", web::get().to(index)),
            )
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

async fn index(data: web::Data<AppState>) -> String{
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[get("/count")]
async fn count(data: web::Data<AppStatWithCounter>) -> impl Responder{
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().body(format!("Request number: {counter}"))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

