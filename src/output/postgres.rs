use std::{process::exit, error::Error, fs::read_to_string};
use log::{warn, error, info};
use async_trait::async_trait;
use sqlx::postgres::PgConnectOptions;

use crate::{output::{OutputChannel, OutputChannelFilter}, config::Config, events::ParsedEvent};

pub struct PostgresOutput {
    pool: sqlx::PgPool,
    filter: OutputChannelFilter,
    skip_rmb_content: bool,
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

        let pool = match parse_connect_options() {
            Ok(options) => {
                match sqlx::PgPool::connect_with(options).await {
                    Ok(pool) => pool,
                    Err(err) => {
                        error!("Error connecting to Postgres: {}", err);
                        exit(1);
                    }
                }
            },
            Err(err) => {
                error!("Error parsing database connection parameters: {}", err);
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
            skip_rmb_content: postgres_config.skip_rmb_content.unwrap_or(false)
        })))
    }

    async fn output(&mut self, event: &ParsedEvent) -> Result<(), Box<dyn Error>> {
        if self.skip_rmb_content && event.category == "rmbpost" && event.data.len() > 1 {
            let mut event = event.clone();
            event.data.resize(1, "".into());
            return self.output(&event).await;
        }

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

fn parse_connect_options() -> Result<PgConnectOptions, Box<dyn Error + Send + Sync>> {
    if let Some(url) = std::env::var("DATABASE_URL").ok() {
        let options: PgConnectOptions = url.parse()?;
        return Ok(options);
    }

    let mut options = PgConnectOptions::new();

    if let Some(host) = std::env::var("DATABASE_HOST").ok() {
        options = options.host(&host);
    }

    if let Some(port) = std::env::var("DATABASE_PORT").ok() {
        options = options.port(port.parse()?);
    }

    if let Some(user) = std::env::var("DATABASE_USER").ok() {
        options = options.username(&user);
    }

    if let Some(name) = std::env::var("DATABASE_NAME").ok() {
        options = options.database(&name);
    }

    if let Some(passfile) = std::env::var("DATABASE_PASSWORD_FILE").ok() {
        let password = read_to_string(passfile)?;
        options = options.password(&password);
    } else if let Some(password) = std::env::var("DATABASE_PASSWORD").ok() {
        options = options.password(&password);
    }

    Ok(options)
}