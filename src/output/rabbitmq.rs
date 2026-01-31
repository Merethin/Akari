use std::{process::exit, error::Error};
use log::{error, warn, info};
use async_trait::async_trait;
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions, ExchangeDeclareOptions}, 
    types::FieldTable, ExchangeKind, BasicProperties
};

use crate::{output::{OutputChannel, OutputChannelFilter}, config::{Config, RabbitMQConfig}, events::ParsedEvent};

pub struct RabbitMQOutput {
    channel: lapin::Channel,
    exchange: String,
    filter: OutputChannelFilter,
}

fn exchange_name(config: &RabbitMQConfig) -> String {
    config.exchange_name.clone().unwrap_or("akari_events".into())
}

#[async_trait]
impl OutputChannel for RabbitMQOutput {
    async fn initialize(config: &Config) -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>> {
        let Some(rmq_config) = &config.output.rmq else {
            return Ok(None);
        };

        if !rmq_config.enabled { return Ok(None); }

        let Ok(url) = std::env::var("RABBITMQ_URL") else {
            error!("RabbitMQ output was enabled but no RABBITMQ_URL was set!");
            exit(1);
        };

        let conn = lapin::Connection::connect(
            &url, lapin::ConnectionProperties::default(),
        ).await?;

        let exchange = exchange_name(rmq_config);
        let channel = conn.create_channel().await?;

        channel.exchange_declare(
            &exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default()
        ).await?;

        channel.confirm_select(ConfirmSelectOptions::default()).await?;

        info!("Created RabbitMQ output exchange named '{}'", exchange);

        Ok(Some(Box::new(Self { 
            channel,
            exchange,
            filter: OutputChannelFilter::new(
                rmq_config.include.clone(), 
                rmq_config.exclude.clone()
            )
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if let Ok(payload) = serde_json::to_string(event) {
            let confirm = self.channel.basic_publish(
                &self.exchange,
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