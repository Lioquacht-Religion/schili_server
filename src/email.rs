// email.rs

use bigdecimal::BigDecimal;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use log::error;
use schili_api::api::SensorType;

use crate::config::EmailConfig;

pub fn send_server_started_email(email_conf: &EmailConfig) {
    let email_text = r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Schili-Sensordaten-Server wird gestartet</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Schili-Sensordaten-Server wird gestartet.</h1>
            <p>Sie sind nun Teil eines sehr exklusiven Klubs 
            und erhalten E-Mails von meinem Schildkröten-Überwachungserver.</p>
            <h2>Empfangen Sie den Spam mit Ehre, Freude und einem offenen Herzen. ❤️</h2>
          </body>
        </html>
        "#;
    send_email_log_error(email_conf, "Server wurde gestartet", email_text.to_owned());
}

pub(crate) fn send_high_measurement_warning_email(
    email_conf: &EmailConfig,
    measure_type: SensorType,
    measurement: &BigDecimal
) {
    let german_name = get_measure_type_name_german(measure_type);
    let german_unit = get_measure_type_unit_german(measure_type);
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Hohe(r) {german_name} wurde gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Hohe(r) {german_name} wurde gerade gemessen!</h1>
            <h1>{german_name}: {} {german_unit}</h1>
          </body>
        </html>
        "#,
        measurement
    );
    send_email_log_error(email_conf, &format!("Hohe(r) {german_name}"), email_text);
}

pub(crate) fn send_low_measurement_warning_email(
    email_conf: &EmailConfig,
    measure_type: SensorType,
    measurement: &BigDecimal,
) {
    let german_name = get_measure_type_name_german(measure_type);
    let german_unit = get_measure_type_unit_german(measure_type);
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Niedrige(r) {german_name} wurde gerade gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Niedrige(r) {german_name} wurde gerade gemessen!</h1>
            <h1>{german_name}: {measurement} {german_unit}</h1>
          </body>
        </html>
        "#
    );
    send_email_log_error(email_conf, &format!("Niedrige(r) {german_name}"), email_text);
}

pub(crate) fn send_measured_strong_increase_email(
    email_conf: &EmailConfig,
    measure_type: SensorType,
    cur_measurement: &BigDecimal,
    prev_measurement: &BigDecimal,
    diff_measurement: &BigDecimal,
) {
    let german_name = get_measure_type_name_german(measure_type);
    let german_unit = get_measure_type_unit_german(measure_type);
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Stark steigende(r) {german_name} wurde gerade gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Stark steigende(r) {german_name} wurde gerade gemessen!</h1>
            <h1>{german_name} ist um {diff_measurement} {german_unit} gestiegen!</h1>
            <h1>Derzeitige(r) {german_name}: {cur_measurement} {german_unit}</h1>
            <h1>Vorherige(r) {german_name}: {prev_measurement} {german_unit}</h1>
          </body>
        </html>
        "#
    );
    send_email_log_error(email_conf, "Stark steigende(r) {german_name}", email_text);
}

pub(crate) fn send_measured_strong_decrease_email(
    email_conf: &EmailConfig,
    measure_type: SensorType,
    cur_measurement: &BigDecimal,
    prev_measurement: &BigDecimal,
    diff_measurement: &BigDecimal,
) {
    let german_name = get_measure_type_name_german(measure_type);
    let german_unit = get_measure_type_unit_german(measure_type);
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Stark sinkende(r) {german_name} wurde gerade gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Stark sinkende(r) {german_name} wurde gerade gemessen!</h1>
            <h1>{german_name} ist um {diff_measurement} {german_unit} gesunken!</h1>
            <h1>Derzeitige(r) {german_name}: {cur_measurement} {german_unit}</h1>
            <h1>Vorherige(r) {german_name}: {prev_measurement} {german_unit}</h1>
          </body>
        </html>
        "#
    );
    send_email_log_error(email_conf, &format!("Stark sinkende(r) {german_name}"), email_text);
}

pub(crate) fn send_email_log_error(email_conf: &EmailConfig, subject: &str, email_text: String) {
    if let Err(e) = send_email(email_conf, subject, email_text) {
        error!("Email could not be sent. Error: {}", e);
    }
}

pub(crate) fn send_email(
    email_conf: &EmailConfig,
    subject: &str,
    email_text: String,
) -> anyhow::Result<()> {
    let email = create_email_message(email_conf, email_text, subject)?;

    let creds = Credentials::new(
        email_conf.smtp_user.to_owned(),
        email_conf.smtp_passw.to_owned(),
    );

    let mailer = SmtpTransport::relay(&email_conf.smtp_server)?
        .credentials(creds)
        .build();

    mailer.send(&email)?;

    Ok(())
}

fn create_email_message(
    email_conf: &EmailConfig,
    email_text: String,
    subject: &str,
) -> anyhow::Result<Message> {
    let mut email = Message::builder().from(Mailbox::new(
        Some(email_conf.author_name.to_owned()),
        email_conf.from_address.parse()?,
    ));
    for address in email_conf.to_addresses.iter() {
        email = email.to(Mailbox::new(None, address.to_owned().parse()?));
    }
    let email = email
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(email_text)?;
    Ok(email)
}

fn get_measure_type_name_german(measure_type: SensorType) -> &'static str{
    match measure_type{
        SensorType::Temperature => "Temperatur",
        SensorType::Humidity => "Luftfeuchtigkeit",
        SensorType::Airpressure => "Luftdruck",
        SensorType::LightIntensity => "Lichtintensität",
        SensorType::BatteryVoltage => "Batteriespannung",
        SensorType::ChipTemperature => "Prozessortemperatur",
        SensorType::Co2 => "CO2-Anteil",
    }
}

fn get_measure_type_unit_german(measure_type: SensorType) -> &'static str{
    match measure_type {
        SensorType::Temperature | SensorType::ChipTemperature
            => "Grad Celsius",
        SensorType::Humidity => "Prozent",
        SensorType::Airpressure => "ppa",
        SensorType::LightIntensity => "ADC",
        SensorType::BatteryVoltage => "Volt",
        SensorType::Co2 => "ppm",
    }
}
