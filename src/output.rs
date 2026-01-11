use std::{io::Write, process::exit};

use lapin::{options::{BasicPublishOptions, ConfirmSelectOptions, ExchangeDeclareOptions}, types::FieldTable, BasicProperties};
use log::{error, info, warn};
use redis_om::{redis, JsonModel};
use file_rotate::{FileRotate, ContentLimit, suffix::{AppendTimestamp, FileLimit}, compression::Compression};

use crate::{config::{Config, RabbitMQConfig, RedisConfig}, events::Event};

pub struct OutputChannels {
    config: Config,
    redis: Option<redis_om::redis::aio::Connection>,
    console: Option<()>,
    file: Option<FileRotate<AppendTimestamp>>,
    rmq: Option<lapin::Channel>,
    postgres: Option<sqlx::PgPool>
}

impl OutputChannels {
    fn new(config: Config) -> Self {
        OutputChannels { config, redis: None, console: None, file: None, rmq: None, postgres: None }
    }
}

fn should_output_event(include: &Option<Vec<String>>, exclude: &Option<Vec<String>>, event: &Event) -> bool {
    if let Some(include_vec) = include
        && !include_vec.contains(&event.category) { return false; }

    if let Some(exclude_vec) = exclude
        && exclude_vec.contains(&event.category) { return false; }

    true
}

fn exchange_name(config: &RabbitMQConfig) -> String {
    config.exchange_name.clone().unwrap_or("akari_events".into())
}

pub async fn open_redis_connection(config: &RedisConfig) -> Result<redis_om::redis::aio::Connection, Box<dyn std::error::Error>> {
    assert!(config.enabled);

    let url = std::env::var("REDIS_URL").unwrap_or_else(|err| {
        error!("Redis output was enabled but no REDIS_URL was set: {err}");
        exit(1);
    });

    let client = redis_om::Client::open(url)?;

    let mut conn = client.get_tokio_connection().await?;

    info!("Connected to Redis");
    
    // We have to create the index manually as redis-om doesn't support it
    let _: () = match redis::cmd("FT.CREATE")
        .arg("idx:event")
        .arg("ON").arg("JSON")
        .arg("PREFIX").arg("1").arg("Event:")
        .arg("SCHEMA")
        .arg("$.id").arg("AS").arg("id").arg("TEXT")
        .arg("$.event").arg("AS").arg("event_id").arg("NUMERIC")
        .arg("$.time").arg("AS").arg("time").arg("NUMERIC")
        .arg("$.actor").arg("AS").arg("actor").arg("TEXT")
        .arg("$.receptor").arg("AS").arg("receptor").arg("TEXT")
        .arg("$.origin").arg("AS").arg("origin").arg("TEXT")
        .arg("$.destination").arg("AS").arg("destination").arg("TEXT")
        .arg("$.category").arg("AS").arg("category").arg("TEXT")
        .arg("$.data[*]").arg("AS").arg("data").arg("TAG").arg("SEPARATOR").arg(",")
        .query_async(&mut conn)
        .await {
            Ok(v) => Ok(v),
            Err(err) => {
                // Ignore this error
                if err.to_string().contains("Index: already exists") { 
                    Ok(())
                }
                else {
                    Err(Box::new(err))
                }
            } 
        }?;

    info!("Redis index created / updated successfully");

    Ok(conn)
}

pub async fn open_rmq_connection(config: &RabbitMQConfig) -> Result<lapin::Connection, Box<dyn std::error::Error>> {
    assert!(config.enabled);

    let url = std::env::var("RABBITMQ_URL").unwrap_or_else(|err| {
        error!("RabbitMQ output was enabled but no RABBITMQ_URL was set: {err}");
        exit(1);
    });

    let conn = lapin::Connection::connect(
        &url, lapin::ConnectionProperties::default(),
    ).await?;

    info!("Connected to RabbitMQ");

    Ok(conn)
}

pub async fn initialize_outputs(config: &Config) -> Result<OutputChannels, Box<dyn std::error::Error>> {
    let mut channels = OutputChannels::new(config.clone());

    if let Some(redis_config) = &config.output.redis
        && redis_config.enabled {
            channels.redis = Some(open_redis_connection(redis_config).await.map_err(|err| {
                error!("Error connecting to Redis: {}", err);

                err
            })?);
        }

    if let Some(rmq_config) = &config.output.rmq
        && rmq_config.enabled {
            let conn = open_rmq_connection(rmq_config).await.map_err(|err| {
                error!("Error connecting to RabbitMQ: {}", err);

                err
            })?;

            let exchange = exchange_name(rmq_config);
            let channel = conn.create_channel().await?;

            channel.exchange_declare(
                &exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default()
            ).await?;

            info!("Created RabbitMQ output exchange named '{}'", exchange);
            
            channel.confirm_select(ConfirmSelectOptions::default()).await?;
            channels.rmq = Some(channel);
        }

    if let Some(console_config) = &config.output.console
        && console_config.enabled {
            channels.console = Some(());
        }

    if let Some(file_config) = &config.output.file
        && file_config.enabled {
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

            channels.file = Some(file);
        }

    if let Some(pg_config) = &config.output.postgres
        && pg_config.enabled {
            let url = std::env::var("DATABASE_URL").unwrap_or_else(|err| {
                error!("Postgres output was enabled but no DATABASE_URL was set: {err}");
                exit(1);
            });

            let pool = sqlx::PgPool::connect(&url).await.map_err(|err| {
                error!("Error connecting to Postgres: {}", err);

                err
            })?;

            channels.postgres = Some(pool);
        }

    Ok(channels)
}

pub async fn process_outputs(channels: &mut OutputChannels, event: &mut Event) 
    -> Result<(), Box<dyn std::error::Error>> {

    if channels.console.is_some() {
        let console_config = channels.config.output.console.clone().unwrap();

        if should_output_event(&console_config.include, &console_config.exclude, event) {
            info!("Event: {}", serde_json::to_string(event).unwrap());
        }
    }

    if let Some(file) = &mut channels.file {
        let file_config = channels.config.output.file.clone().unwrap();

        if should_output_event(&file_config.include, &file_config.exclude, event) {
            writeln!(file, "{}", serde_json::to_string(event).unwrap())?;
        }
    }

    if let Some(channel) = &mut channels.rmq {
        let rmq_config = channels.config.output.rmq.clone().unwrap();

        if should_output_event(&rmq_config.include, &rmq_config.exclude, event) {
            let payload = serde_json::to_string(event).unwrap();

            let confirm = channel
                .basic_publish(
                    &exchange_name(&rmq_config),
                    &event.category,
                    BasicPublishOptions::default(),
                    payload.as_bytes(),
                    BasicProperties::default(),
                )
                .await?
                .await?;

            if !confirm.is_ack() {
                warn!("Failed to send event '{:?}' to RabbitMQ - {:?}", event, confirm);
            }
        }
    }


    if let Some(connection) = &mut channels.redis {
        let redis_config = channels.config.output.redis.clone().unwrap();

        if should_output_event(&redis_config.include, &redis_config.exclude, event) {
            event.save(connection).await?;
        }
    }

    if let Some(pool) = &channels.postgres {
        let pg_config = channels.config.output.postgres.clone().unwrap();

        if should_output_event(&pg_config.include, &pg_config.exclude, event) {
            if event.event == -1 {
                if let Some(system_table) = pg_config.system_table_name {
                    let result = sqlx::query(
                    &format!("INSERT INTO {} (time, category, data) VALUES ($1, $2, $3)", system_table)
                    ).bind(event.time as i64)
                    .bind(event.category.clone())
                    .bind(event.data.clone())
                    .execute(pool).await;

                    if result.is_err() {
                        warn!("Failed to save event '{:?}' to Postgres database - {:?}", event, result);
                    }
                }
            } else if let Some(table) = pg_config.table_name {
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
                .execute(pool).await;

                if result.is_err() {
                    warn!("Failed to save event '{:?}' to Postgres database - {:?}", event, result);
                }
            }
        }
    }

    Ok(())
}