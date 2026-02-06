use std::{process::exit, error::Error};
use log::{warn, error, info};
use async_trait::async_trait;

use crate::{output::{OutputChannel, OutputChannelFilter}, config::Config, events::ParsedEvent};

pub struct PostgresOutput {
    pool: sqlx::PgPool,
    filter: OutputChannelFilter,
}

const TABLE_NAME: &'static str = "akari_events";
const SYSTEM_TABLE_NAME: &'static str = "akari_system_events";

#[async_trait]
impl OutputChannel for PostgresOutput {
    async fn initialize(config: &Config) -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>> {
        let Some(postgres_config) = &config.output.postgres else {
            return Ok(None);
        };

        if !postgres_config.enabled { return Ok(None); }

        let Ok(url) = std::env::var("DATABASE_URL") else {
            error!("Postgres output was enabled but no DATABASE_URL was set!");
            exit(1);
        };

        let pool = match sqlx::PgPool::connect(&url).await {
            Ok(pool) => pool,
            Err(err) => {
                error!("Error connecting to Postgres: {}", err);
                exit(1);
            }
        };

        sqlx::migrate!().run(&pool).await?;

        info!("Connected to Postgres database and saving to table '{}'", TABLE_NAME);

        Ok(Some(Box::new(Self {
            pool,
            filter: OutputChannelFilter::new(
                postgres_config.include.clone(), 
                postgres_config.exclude.clone()
            ),
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if event.event == -1 {
            let result = sqlx::query(
                &format!("INSERT INTO {} (time, category, data) VALUES ($1, $2, $3)", SYSTEM_TABLE_NAME)
            ).bind(event.time as i64)
            .bind(event.category.clone())
            .bind(event.data.clone())
            .execute(&self.pool).await;

            if result.is_err() {
                warn!("Failed to save event '{:?}' to Postgres database - {:?}", event, result);
            }
        } else {
            let result = sqlx::query(
                &format!("INSERT INTO {} (event, time, actor, receptor, origin, destination, category, data)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING", TABLE_NAME)
            ).bind(event.event)
            .bind(event.time as i64)
            .bind(event.actor.clone())
            .bind(event.receptor.clone())
            .bind(event.origin.clone())
            .bind(event.destination.clone())
            .bind(event.category.clone())
            .bind(event.data.clone())
            .execute(&self.pool).await;

            if result.is_err() {
                warn!("Failed to save event '{:?}' to Postgres database - {:?}", event, result);
            }
        }

        Ok(())
    }

    fn get_filter(&self) -> &OutputChannelFilter {
        &self.filter
    }
}