use std::{process::exit, error::Error, fs::read_to_string};
use log::{error, warn, info};
use async_trait::async_trait;
use lapin::{
    BasicProperties, ExchangeKind, options::{BasicPublishOptions, ConfirmSelectOptions, ExchangeDeclareOptions}, types::FieldTable, uri::AMQPUri
};

use crate::{output::{OutputChannel, OutputChannelFilter}, config::Config, events::ParsedEvent};

pub struct RabbitMQOutput {
    channel: lapin::Channel,
    filter: OutputChannelFilter,
}

const EXCHANGE_NAME: &'static str = "akari_events";

#[async_trait]
impl OutputChannel for RabbitMQOutput {
    async fn initialize(config: &Config) -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>> {
        let Some(rmq_config) = &config.output.rmq else {
            return Ok(None);
        };

        if !rmq_config.enabled { return Ok(None); }

        let conn = match parse_connection_uri() {
            Ok(uri) => {
                match lapin::Connection::connect_uri(
                    uri, lapin::ConnectionProperties::default(),
                ).await {
                    Ok(conn) => conn,
                    Err(err) => {
                        error!("Error connecting to RabbitMQ: {}", err);
                        exit(1);
                    }
                }
            },
            Err(err) => {
                error!("Error parsing RabbitMQ connection parameters: {}", err);
                exit(1);
            }
        };

        let channel = conn.create_channel().await?;

        channel.exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default()
        ).await?;

        channel.confirm_select(ConfirmSelectOptions::default()).await?;

        info!("Created RabbitMQ output exchange named '{}'", EXCHANGE_NAME);

        Ok(Some(Box::new(Self { 
            channel,
            filter: OutputChannelFilter::new(
                rmq_config.include.clone(), 
                rmq_config.exclude.clone()
            )
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if let Ok(payload) = serde_json::to_string(event) {
            let confirm = self.channel.basic_publish(
                EXCHANGE_NAME,
                &event.category,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default(),
            ).await?.await?;

            if !confirm.is_ack() {
                warn!("Failed to send event '{:?}' to RabbitMQ - {:?}", event, confirm);
            }
        }

        Ok(())
    }

    fn get_filter(&self) -> &OutputChannelFilter {
        &self.filter
    }
}

fn parse_connection_uri() -> Result<AMQPUri, Box<dyn Error + Send + Sync>> {
    if let Some(url) = std::env::var("RABBITMQ_URL").ok() {
        let uri: AMQPUri = url.parse()?;
        return Ok(uri);
    }

    let mut uri = AMQPUri::default();

    if let Some(host) = std::env::var("RABBITMQ_HOST").ok() {
        uri.authority.host = host;
    }

    if let Some(port) = std::env::var("RABBITMQ_PORT").ok() {
        uri.authority.port = port.parse()?;
    }

    if let Some(user) = std::env::var("RABBITMQ_USER").ok() {
        uri.authority.userinfo.username = user;
    }

    if let Some(passfile) = std::env::var("RABBITMQ_PASSWORD_FILE").ok() {
        let password = read_to_string(passfile)?;
        uri.authority.userinfo.password = password;
    } else if let Some(password) = std::env::var("RABBITMQ_PASSWORD").ok() {
        uri.authority.userinfo.password = password;
    }

    Ok(uri)
}