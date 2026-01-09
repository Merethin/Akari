mod parser;
mod events;
mod net;
mod patterns;
mod config;
mod output;
mod worker;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{env, process::exit};
use config_file::FromConfigFile;
use crossbeam::channel::Sender;
use log::{info, warn, error, STATIC_MAX_LEVEL};
use simplelog::{Config as LogConfig, TermLogger, TerminalMode, ColorChoice};

use crate::config::Config;
use crate::net::{Connection, ExponentialBackoff, ReadMessageState};
use crate::output::initialize_outputs;
use crate::worker::spawn_work_threads;
use crate::events::{Message, ServerEvent, SystemEvent};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_PATH: &str = "config/akari.toml";

static SEQUENCE_ID: AtomicUsize = AtomicUsize::new(0);

fn sequenced(msg: Message) -> (usize, Message) {
    let seq_id = SEQUENCE_ID.fetch_add(1, Ordering::Relaxed);

    (seq_id, msg)
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect(
        "Current system time should be later than the Unix epoch"
    ).as_secs()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(
        STATIC_MAX_LEVEL, LogConfig::default(), TerminalMode::Stderr, ColorChoice::Auto
    )?;

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
    sender: Sender<(usize, Message)>,
    backoff: &mut ExponentialBackoff<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_event_id: Option<i64> = None;

    loop {
        let mut connection = connect_retry_loop(&config, user_agent, backoff).await;

        sender.send(sequenced(
            Message::System(SystemEvent::connection_initialized(now()))
        )).unwrap_or_else(|err| {
            error!("Failed to send system event to worker: {err}");
        });

        let mut state = ReadMessageState::default();
        let mut last_event_time = Instant::now();

        loop {
            let should_continue = read_and_parse_messages(&mut connection, &mut state)
                .await?.into_iter().map(|message| {
                match message {
                    ReadMessageResult::Message(event) => {
                        let current_id: i64 = event.id.parse().unwrap_or(-1);

                        if Some(current_id) == last_event_id {
                            return true; // Skip duplicate event
                        }

                        if let Some((last_id, events_missed)) = detect_missed_events(last_event_id, current_id) {
                            sender.send(sequenced(
                                Message::System(
                                    SystemEvent::events_missed(
                                        now(), events_missed, last_id, current_id
                                    )
                                )
                            )).unwrap_or_else(|err| {
                                error!("Failed to send system event to worker: {err}");
                            });
                        }

                        sender.send(sequenced(Message::Server(event))).unwrap_or_else(|err| {
                            error!("Failed to send server event to worker: {err}");
                        });

                        last_event_time = Instant::now();
                        last_event_id = Some(current_id);

                        return true;
                    },
                    ReadMessageResult::NoMessage => {
                        let elapsed = Instant::now().duration_since(last_event_time);
                        if elapsed.as_secs() > 30 {
                            warn!("No events in the last 30 seconds, dropping connection and reconnecting");
                            return false;
                        }

                        return true;
                    },
                    ReadMessageResult::ResponseError => {
                        return false;
                    }
                }
            }).last().unwrap_or(true);

            if !should_continue { break; }
        }

        drop(connection);

        sender.send(sequenced(Message::System(
            SystemEvent::connection_dropped(now(), last_event_id.unwrap_or(-1))
        ))).unwrap_or_else(|err| {
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

async fn connect_retry_loop(
    config: &Config,
    user_agent: &String, 
    backoff: &mut ExponentialBackoff<'_>
) -> Connection {
    loop {
        match net::connect(&config.input, user_agent).await {
            Ok(conn) => {
                backoff.reset();
                return conn;
            },
            Err(err) => {
                warn!("Error while connecting to www.nationstates.net: {}", err);
                info!("Attempting to reconnect in {} seconds", backoff.delay());
                backoff.wait().await;
            }
        }
    }
}

enum ReadMessageResult {
    NoMessage,
    Message(ServerEvent),
    ResponseError
}

async fn read_and_parse_messages(
    connection: &mut Connection,
    state: &mut ReadMessageState,
) -> Result<Vec<ReadMessageResult>, Box<dyn std::error::Error>> {
    let messages = match net::read_messages(connection, state).await {
        Ok(v) => Ok(v),
        Err(err) => {
            let e = match err.downcast::<reqwest::Error>() {
                Ok(req_err) => {
                    if req_err.is_timeout() {
                        warn!("Read from NationStates timed out: {}", req_err);
                        return Ok(vec![ReadMessageResult::ResponseError]);
                    }

                    if req_err.is_decode() {
                        warn!("Error decoding response from NationStates: {}", req_err);
                        return Ok(vec![ReadMessageResult::ResponseError]);
                    }

                    req_err
                },
                Err(original_err) => {
                    original_err
                }
            };

            Err(e)
        }
    }?;

    Ok(messages.iter().map(|message| {match parser::parse_message(message.as_str()) {
        Some(event) => ReadMessageResult::Message(event),
        None => ReadMessageResult::NoMessage,
    }}).collect())
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