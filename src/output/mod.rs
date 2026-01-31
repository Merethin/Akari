mod console;
mod file;
mod postgres;
mod rabbitmq;

use std::error::Error;
use crate::{config::Config, events::ParsedEvent};
use async_trait::async_trait;
use std::collections::HashSet;

use console::ConsoleOutput;
use file::FileOutput;
use postgres::PostgresOutput;
use rabbitmq::RabbitMQOutput;

#[async_trait]
pub trait OutputChannel: Send {
    async fn initialize(config: &Config) 
        -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>>
    where
        Self: Sized;

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>>;

    fn get_filter(&self) -> &OutputChannelFilter;
}

pub struct OutputChannelFilter {
    include_list: Option<HashSet<String>>,
    exclude_list: Option<HashSet<String>>,
}

impl OutputChannelFilter {
    pub fn new(
        include: Option<Vec<String>>, exclude: Option<Vec<String>>
    ) -> Self {
        Self {
            include_list: include.map(|v| HashSet::from_iter(v.into_iter())),
            exclude_list: exclude.map(|v| HashSet::from_iter(v.into_iter())),
        }
    }

    pub fn should_output_event(&self, event: &ParsedEvent) -> bool {
        if let Some(include_list) = &self.include_list
            && !include_list.contains(&event.category) { return false; }

        if let Some(exclude_list) = &self.exclude_list
            && exclude_list.contains(&event.category) { return false; }

        true
    }
}

pub async fn initialize_outputs(
    config: &Config
) -> Result<Vec<Box<dyn OutputChannel>>, Box<dyn Error>> {
    let mut channels = Vec::new();

    channels.push(RabbitMQOutput::initialize(config).await?);
    channels.push(ConsoleOutput::initialize(config).await?);
    channels.push(FileOutput::initialize(config).await?);
    channels.push(PostgresOutput::initialize(config).await?);

    Ok(channels.into_iter().flatten().collect())
}

pub async fn process_outputs(
    channels: &mut Vec<Box<dyn OutputChannel>>, event: &mut ParsedEvent
) -> Result<(), Box<dyn Error>> {
    for channel in channels {
        if channel.get_filter().should_output_event(event) {
            channel.output(event).await?;
        }
    }

    Ok(())
}