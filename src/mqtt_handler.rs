// mqtt_handler.rs

use std::{str::Chars, time::Duration};

use log::{error, info};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Publish, QoS, StateError};
use schili_api::mq_topics::{
    TOPICS, chip_temperature_topic, sensor_airpressure_topic, sensor_battery_voltage_topic, sensor_co2_topic, sensor_error_topic, sensor_humidity_topic, sensor_lightintensity_topic, sensor_measurements_bundle_topic, sensor_temperature_topic
};
use sqlx::{Pool, Postgres};
use tokio::io;

use crate::{config::Config, database, service};

static UUID: &str = "42";

pub async fn start_mq_client(app_config: &Config) {
    let mut mqttoptions = MqttOptions::new(
        &app_config.mqtt.broker_id,
        &app_config.mqtt.host,
        app_config.mqtt.port,
    );
    mqttoptions.set_credentials(&app_config.mqtt.username, &app_config.mqtt.passw);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let pool = database::create_db_pool().await;

    let _handle = actix_rt::spawn(async move {
        loop{
            subscribe_to_topics(&client).await;
            handle_mq_events(&mut eventloop, &pool).await;
        }
    });
}

struct TopicRouter{
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

struct NodeId(usize);

enum SegmentKind{
    Str(String),
    Enum(Vec<String>),
    Var,
}

enum SegmentVal{
    Str,
    Enum(String),
    Var(String),
}

struct Node{
    value: String,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

struct Cursor<'a>{
    chars: Chars<'a>,
    len_remaining: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenKind{
    Ident,
    OpenSquareBrace,
    CloseSquareBrace,
    OpenCurlyBrace,
    CloseCurlyBrace,
    Pipe,
    Slash,
    Whitespace,
    Unknown,
    EOF,
}

const EOF_CHAR: char = '\0';

struct Token{
    kind: TokenKind,
    len: u32,
}

struct Parser<'a>{
    cursor: Cursor<'a>,
    last_token: Token,
}

impl<'a> Parser<'a>{
    fn new(route: &'a str) -> Parser<'a>{
        Parser { 
            cursor: Cursor::new(route), 
            last_token: Token { kind: TokenKind::EOF, len: 0 }
        }
    }

    fn advance_parser(&mut self, route: &str) -> Option<SegmentKind>{
        let mut cursor = Cursor::new(route);
        let token = cursor.advance_token();

        Some(match token.kind {
            TokenKind::Ident => SegmentKind::Str(route[..(token.len as usize)].to_owned()),
            TokenKind::OpenSquareBrace => todo!(),
            TokenKind::CloseSquareBrace => todo!(),
            TokenKind::OpenCurlyBrace => todo!(),
            TokenKind::CloseCurlyBrace => todo!(),
            TokenKind::Pipe => todo!(),
            TokenKind::Slash => todo!(),
            TokenKind::Whitespace => todo!(),
            TokenKind::Unknown => todo!(),
            TokenKind::EOF => return None,
        })

        //token = Cursor::new(&route[(token.len as usize)..]).advance_token();
    }

    fn parse_ident(){
    }

    fn parse_enum(){
    }

    fn parse_var(){
    }
}

impl<'a> Cursor<'a>{
    fn new(src: &'a str) -> Cursor<'a>{
        Self { 
            len_remaining: src.len(),
            chars: src.chars(),
        }
    }

    fn is_eof(&self) -> bool{
        self.chars.as_str().is_empty()
    }

    fn as_str(&self) -> &'a str{
        self.chars.as_str()
    }

    fn first(&self) -> char{
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn second(&self) -> char{
        let mut chars = self.chars.clone();
        chars.next();
        chars.next()
            .unwrap_or(EOF_CHAR)
    }

    fn bump(&mut self) -> Option<char>{
        self.chars.next()
    }

    fn eat_while(&mut self, predicate: impl Fn(char) -> bool){
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    fn eat_until(&mut self, byte: u8){
        let mut bytes = self.as_str().bytes().enumerate();
        self.chars = loop {
            match bytes.next() {
                Some((i, cur_byte)) => if cur_byte == byte{
                    break self.as_str()[i..].chars();
                },
                None => break "".chars(),
            }
        };
    }

    fn white_space(&mut self) -> TokenKind{
        self.eat_while(char::is_whitespace);
        TokenKind::Whitespace
    }

    fn is_ident_start(c: char) -> bool{
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_ident_continue(c: char) -> bool{
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    }

    fn ident(&mut self) -> TokenKind{
        self.eat_while(Self::is_ident_continue);
        TokenKind::Ident
    }

    fn pos_within_token(&self) -> u32{
        (self.len_remaining - self.chars.as_str().len()) as u32
    }

    fn advance_token(&mut self) -> Token{
        let Some(first_char) = self.chars.next() else {
            return Token{kind: TokenKind::EOF, len: 0};
        };

        let kind: TokenKind = match first_char{
            '/' => TokenKind::Slash,
            '[' => TokenKind::OpenSquareBrace,
            ']' => TokenKind::CloseSquareBrace,
            '{' => TokenKind::OpenCurlyBrace,
            '}' => TokenKind::CloseCurlyBrace,
            '|' => TokenKind::Pipe,
            c if c.is_whitespace() => self.white_space(),
            c if Self::is_ident_start(c) => self.ident(),
            _ => TokenKind::Unknown,
        };
        Token { kind, len: self.pos_within_token() }
    }
}

impl TopicRouter{
    fn new() -> Self{
        Self { nodes: Vec::new(), root: None }
    }

    fn parse_route(&mut self, route: &str){
        let mut last_token = Cursor::new(route).advance_token();
        while last_token.kind != TokenKind::EOF{

            last_token = Cursor::new(&route[(last_token.len as usize)..]).advance_token();
        }
    }

    fn add_route(&mut self, route: &str){
    }
}


// {var1}/routepart1/routepart2/[enumval1|enumval2|enumval3]/routepart3

#[cfg(test)]
mod test{
    #[test]
    fn url_router_test(){

    }
}

async fn subscribe_to_topics(client: &AsyncClient) {
    client
        .subscribe(chip_temperature_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_temperature_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_humidity_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_airpressure_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_lightintensity_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_battery_voltage_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_co2_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_measurements_bundle_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
    client
        .subscribe(sensor_error_topic(UUID), QoS::AtLeastOnce)
        .await
        .unwrap();
}

struct DisconnectOccured;

async fn handle_mq_events(eventloop: &mut EventLoop, db_pool: &Pool<Postgres>) -> DisconnectOccured{
    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                if let Err(es) = handle_publish(db_pool, &publish).await {
                    for e in es{
                        error!(
                            "An error occured while trying to process published messages: error: {e}"
                        );
                    }
                };
            }
            Ok(_event) => {
                continue;
            }
            Err(e) => {
                error!("Error received = {:?}", e);
                if let rumqttc::ConnectionError::MqttState(StateError::Io(e)) = e {
                    if let io::ErrorKind::ConnectionAborted = e.kind(){
                        return DisconnectOccured;
                    }
                }
            }
        }
    }
}

async fn handle_publish(pool: &Pool<Postgres>, publish: &Publish) -> anyhow::Result<(), Vec<anyhow::Error>> {
    info!("Publish received for topic: {}", &publish.topic);
    let mut errors: Vec<anyhow::Error> = Vec::new();
    if publish.topic.contains(&TOPICS.chip_temp) {
        let chip_temp = extract_sensor_simple_measurement(publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_chip_temperature(pool, &chip_temp).await
            .map_err(|e| errors.push(e));
    }
    if publish.topic.contains(&TOPICS.temp) {
        let sens_temps = extract_sensor_simple_measurement(&publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_temperature_w_sensor(pool, &sens_temps).await
            .map_err(|e| errors.push(e));
    }
    if publish.topic.contains(&TOPICS.humidity) {
        let sens_hums = extract_sensor_simple_measurement(&publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_humidity(pool, &sens_hums).await
            .map_err(|e| errors.push(e));
    }
    if publish.topic.contains(&TOPICS.air_pressure) {
        let sens_hums = extract_sensor_simple_measurement(&publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_airpressure(pool, &sens_hums).await
            .map_err(|e| errors.push(e));
    }
    if publish.topic.contains(&TOPICS.light_intensity) {
        let sens_hums = extract_sensor_simple_measurement(&publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_airpressure(pool, &sens_hums).await
            .map_err(|e| errors.push(e));
    }

    if publish.topic.contains(&TOPICS.battery_voltage) {
        let sens_battv = extract_sensor_simple_measurement(&publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_battery_voltage(pool, &sens_battv).await
            .map_err(|e| errors.push(e));
    }
    if publish.topic.contains(&TOPICS.co2) {
        let sens_co2 = extract_sensor_co2(&publish)
            .map_err(|e| vec![e])?;
        if let Err(e) = service::insert_co2(pool, &sens_co2).await {
            error!("Could not insert co2 from mq publish. error: {}", e);
        }

        info!(
            "sensor temps: {}",
            serde_json::to_string(&sens_co2).unwrap()
        );
    }
    if publish.topic.contains(&TOPICS.measurement_bundle) {
        let mut sensor = extract_sensor_measurement_bundle(&publish)
            .map_err(|e| vec![e])?;
        service::insert_bundled_measurements(pool, &mut sensor).await?;
    }
    if publish.topic.contains(&TOPICS.error) {
        let sensor_error= extract_sensor_error(&publish)
            .map_err(|e| vec![e])?;
        let _ = service::insert_sensor_error(pool, &sensor_error).await
            .map_err(|e| errors.push(e));
    }
    if errors.is_empty(){
        Ok(())
    }
    else{
        Err(errors)
    }
}

fn extract_sensor_simple_measurement(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleSimpleMeasure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_measurement_bundle(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorTypedSimpleMeasurements> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    info!("measure bundle: {}", &json_str);
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_co2(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorSingleCo2Measure> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}

fn extract_sensor_error(
    publish: &Publish,
) -> anyhow::Result<schili_api::api::SensorError> {
    let json_str: String = String::from_utf8(publish.payload.to_vec())?;
    Ok(serde_json::from_str(&json_str)?)
}
