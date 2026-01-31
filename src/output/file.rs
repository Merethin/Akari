use file_rotate::{ContentLimit, FileRotate, compression::Compression, suffix::{AppendTimestamp, FileLimit}};
use std::{process::exit, error::Error, io::Write};
use log::error;
use async_trait::async_trait;

use crate::{output::{OutputChannel, OutputChannelFilter}, config::Config, events::ParsedEvent};

pub struct FileOutput {
    file: FileRotate<AppendTimestamp>,
    filter: OutputChannelFilter,
}

#[async_trait]
impl OutputChannel for FileOutput {
    async fn initialize(config: &Config) -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>> {
        let Some(file_config) = &config.output.file else {
            return Ok(None);
        };

        if !file_config.enabled { return Ok(None); }

        let file = 
            FileRotate::new(file_config.path.clone().unwrap_or_else(|| {
                error!("File output was enabled but no path was set");
                exit(1);
            }), 
            AppendTimestamp::default(
                file_config.maxfiles.map_or(
                    FileLimit::Unlimited, |limit| FileLimit::MaxFiles(limit)
                )
            ), 
            ContentLimit::Lines(
                file_config.threshold.unwrap_or(500) * 1000
            ), 
            Compression::OnRotate(0),
            None);

        Ok(Some(Box::new(Self { 
            file,
            filter: OutputChannelFilter::new(
                file_config.include.clone(), 
                file_config.exclude.clone()
            )
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if let Ok(serialized) = serde_json::to_string(event) {
            writeln!(&mut self.file, "{}", serialized)?;
        }

        Ok(())
    }

    fn get_filter(&self) -> &OutputChannelFilter {
        &self.filter
    }
}