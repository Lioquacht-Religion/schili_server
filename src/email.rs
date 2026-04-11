// email.rs

use bigdecimal::BigDecimal;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use log::error;

use crate::config::EmailConfig;

pub fn send_server_started_email(
    email_conf: &EmailConfig,
) {
    let email_text = 
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Schili-Sensordaten-Server wird gestartet</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Schili-Sensordaten-Server wird gestartet.</h1>
            <p>Sie sind nun Teil eines sehr exclusiven Klubs 
            und erhalten E-Mails von meinem Schildkröten-Überwachungserver.</p>
            <h2>Empfangen Sie den Spam mit Ehre, Freude und einem offenen Herzen. ❤️</h2>
          </body>
        </html>
        "#
    ;
    send_email_log_error(email_conf, "Server wurde gestartet", email_text.to_owned());
}

pub fn send_low_batt_voltage_warning_email(
    email_conf: &EmailConfig,
    batt_voltage: &BigDecimal,
) {
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Niedrige Batteriespannung wurde gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Niedrige Batteriespannung wurde gemessen!</h1>
            <h1>Batteriespannung: {} Volt</h1>
          </body>
        </html>
        "#,
        batt_voltage
    );
    send_email_log_error(email_conf, "Niedrige Batteriespannung", email_text);
}

pub fn send_high_chip_temp_warning_email(
    email_conf: &EmailConfig,
    temp_celsius: &BigDecimal,
) {
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Hohe Prozessortemperatur wurde gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Hohe Prozessortemperatur wurde gerade gemessen!</h1>
            <h1>Temperatur: {} Grad Celsius</h1>
          </body>
        </html>
        "#,
        temp_celsius
    );
    send_email_log_error(email_conf, "Hohe Prozessortemperatur", email_text);
}

pub fn send_high_temp_warning_email(
    email_conf: &EmailConfig,
    temp_celsius: &BigDecimal,
) {
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Hohe temperatur wurde gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Hohe temperatur wurde gerade gemessen!</h1>
            <h1>Temperatur: {} Grad Celsius</h1>
          </body>
        </html>
        "#,
        temp_celsius
    );
    send_email_log_error(email_conf, "Hohe Temperatur", email_text);
}

pub fn send_low_temp_warning_email(
    email_conf: &EmailConfig,
    temp_celsius: &BigDecimal,
) {
    let email_text = format!(
        r#"
        <!DOCTYPE html>
        <html>
          <head>
            <title>Niedrige temperatur wurde gerade gemessen!</title>
            <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          </head>
          <body>
            <h1>Niedrige temperatur wurde gerade gemessen!</h1>
            <h1>Temperatur: {} Grad Celsius</h1>
          </body>
        </html>
        "#,
        temp_celsius
    );
    send_email_log_error(email_conf, "Niedrige Temperatur", email_text);
}

pub fn send_email_log_error(email_conf: &EmailConfig, subject: &str, email_text: String) {
    if let Err(e) = send_email(email_conf, subject, email_text){
        error!("Email could not be sent. Error: {}", e);
    }
}

pub fn send_email(email_conf: &EmailConfig, subject: &str, email_text: String) -> anyhow::Result<()> {
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

fn create_email_message(email_conf: &EmailConfig, email_text: String, subject: &str) -> anyhow::Result<Message>{
    let mut email = Message::builder()
        .from(Mailbox::new(
            Some(email_conf.author_name.to_owned()),
            email_conf.from_address.parse()?,
        ));
    for address in email_conf.to_addresses.iter(){
        email = email
        .to(Mailbox::new(
                    None,
            address.to_owned().parse()?,
        ));
    }
    let email = email
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(email_text)?;
    Ok(email)
}
