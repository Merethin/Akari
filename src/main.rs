mod parser;
mod events;
mod net;
mod config;
mod output;
mod worker;

use std::{env, process::exit, error::Error, time::Instant};
use config_file::FromConfigFile;
use crossbeam::channel::Sender;
use log::{info, warn, error, STATIC_MAX_LEVEL};
use simplelog::{Config as LogConfig, TermLogger, TerminalMode, ColorChoice};

use crate::config::Config;
use crate::net::{Connection, MessageResult, ExponentialBackoff};
use crate::output::initialize_outputs;
use crate::worker::spawn_work_threads;
use crate::events::{SystemEvent, SequencedEvent};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_PATH: &str = "config/akari.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    TermLogger::init(
        STATIC_MAX_LEVEL, LogConfig::default(), TerminalMode::Stderr, ColorChoice::Auto
    )?;

    dotenv::dotenv().ok();

    let user_agent = read_user_agent();
    info!("Running with user agent '{}'", user_agent);

    let config = read_config();
    let outputs = initialize_outputs(&config).await?;
    let sender = spawn_work_threads(outputs, config.input.workers);
    let mut backoff = ExponentialBackoff::new(&[60, 120, 240, 960, 1800]);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down...");
        },
        _ = main_loop(&user_agent, config, sender, &mut backoff) => {},
    }

    Ok(())
}

async fn main_loop(
    user_agent: &String,
    config: Config,
    sender: Sender<SequencedEvent>,
    backoff: &mut ExponentialBackoff<'_>,
) -> Result<(), Box<dyn Error>> {
    let mut last_event_id: Option<i64> = None;

    loop {
        let mut connection = Connection::connect(&config.input.url, user_agent, backoff).await;

        sender.send(SystemEvent::connection_initialized()).unwrap_or_else(|err| {
            error!("Failed to send system event to worker: {err}");
        });

        let mut last_event_time = Instant::now();

        loop {
            match connection.read_messages().await? {
                MessageResult::Messages(messages) => {
                    for event in messages {
                        let current_id: i64 = event.id.parse().unwrap_or(-1);

                        if Some(current_id) == last_event_id {
                            continue;
                        }

                        if let Some((last_id, events_missed)) = detect_missed_events(last_event_id, current_id) {
                            sender.send(SystemEvent::events_missed(
                                events_missed, last_id, current_id
                            )).unwrap_or_else(|err| {
                                error!("Failed to send system event to worker: {err}");
                            });
                        }

                        sender.send(SequencedEvent::wrap_server(event)).unwrap_or_else(|err| {
                            error!("Failed to send server event to worker: {err}");
                        });

                        last_event_time = Instant::now();
                        last_event_id = Some(current_id);
                    }
                },
                MessageResult::NoMessages => {
                    let elapsed = Instant::now().duration_since(last_event_time);
                    if elapsed.as_secs() > 30 {
                        warn!("No events in the last 30 seconds, dropping connection and reconnecting");
                        break;
                    }
                },
                MessageResult::ResponseError => {
                    break;
                }
            }
        }

        drop(connection);

        sender.send(
            SystemEvent::connection_dropped(last_event_id.unwrap_or(-1))
        ).unwrap_or_else(|err| {
            error!("Failed to send system event to worker: {err}");
        });

        info!("Attempting to reconnect");
    }
}

fn read_user_agent() -> String {
    let user = match env::var("NS_USER_AGENT") {
        Ok(user) => user,
        Err(err) => match err {
            env::VarError::NotPresent => {
                error!("No user agent provided, please set the NS_USER_AGENT environment variable to your main nation name");
                exit(1);
            },
            env::VarError::NotUnicode(_) => {
                error!("User agent is not valid unicode");
                exit(1);
            }
        }
    };

    format!("Akari/{} by Merethin, in use by {}", VERSION, user)
}

fn read_config() -> Config {
    Config::from_config_file(CONFIG_PATH).unwrap_or_else(|e| {
        warn!("Failed to load config file: {} - loading default values", e);
        Config::default()
    })
}

fn detect_missed_events(last_event_id: Option<i64>, current_id: i64) -> Option<(i64, i64)> {
    if let Some(last_id) = last_event_id {
        let difference = current_id - (last_id + 1);
        if difference > 0 {
            warn!(
                "Missed {} NationStates events (from {} to {})",
                difference, last_id, current_id
            );

            return Some((last_id, difference));
        }
    }

    None
}