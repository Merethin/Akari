use std::error::Error;
use log::info;
use async_trait::async_trait;

use crate::{output::{OutputChannel, OutputChannelFilter}, config::Config, events::ParsedEvent};

pub struct ConsoleOutput {
    filter: OutputChannelFilter,
}

#[async_trait]
impl OutputChannel for ConsoleOutput {
    async fn initialize(config: &Config) -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>> {
        let Some(console_config) = &config.output.console else {
            return Ok(None);
        };

        if !console_config.enabled { return Ok(None); }

        info!("Console output initialized");

        Ok(Some(Box::new(Self { 
            filter: OutputChannelFilter::new(
                console_config.include.clone(), 
                console_config.exclude.clone()
            )
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if let Ok(serialized) = serde_json::to_string(event) {
            info!("Event: {}", serialized);
        }

        Ok(())
    }

    fn get_filter(&self) -> &OutputChannelFilter {
        &self.filter
    }
}