use std::{process::exit, error::Error};
use log::{warn, error};
use async_trait::async_trait;

use crate::{output::{OutputChannel, OutputChannelFilter}, config::Config, events::ParsedEvent};

pub struct PostgresOutput {
    pool: sqlx::PgPool,
    filter: OutputChannelFilter,
    table_name: Option<String>,
    system_table_name: Option<String>,
}

#[async_trait]
impl OutputChannel for PostgresOutput {
    async fn initialize(config: &Config) -> Result<Option<Box<dyn OutputChannel>>, Box<dyn Error>> {
        let Some(postgres_config) = &config.output.postgres else {
            return Ok(None);
        };

        if !postgres_config.enabled { return Ok(None); }

        let url = std::env::var("DATABASE_URL").unwrap_or_else(|err| {
            error!("Postgres output was enabled but no DATABASE_URL was set: {err}");
            exit(1);
        });

        let pool = sqlx::PgPool::connect(&url).await.map_err(|err| {
            error!("Error connecting to Postgres: {}", err);

            err
        })?;

        Ok(Some(Box::new(Self {
            pool,
            filter: OutputChannelFilter::new(
                postgres_config.include.clone(), 
                postgres_config.exclude.clone()
            ),
            table_name: postgres_config.table_name.clone(),
            system_table_name: postgres_config.system_table_name.clone(),
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if event.event == -1 {
            if let Some(system_table) = &self.system_table_name {
                let result = sqlx::query(
                &format!("INSERT INTO {} (time, category, data) VALUES ($1, $2, $3)", system_table)
                ).bind(event.time as i64)
                .bind(event.category.clone())
                .bind(event.data.clone())
                .execute(&self.pool).await;

                if result.is_err() {
                    warn!("Failed to save event '{:?}' to Postgres database - {:?}", event, result);
                }
            }
        } else if let Some(table) = &self.table_name {
            let result = sqlx::query(
            &format!("INSERT INTO {} (event, time, actor, receptor, origin, destination, category, data)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING", table)
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