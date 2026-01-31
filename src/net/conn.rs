use std::{fmt, collections::VecDeque, time::Duration, error::Error, pin::Pin};

use bytes::Bytes;
use reqwest::{Method, StatusCode};
use log::{warn, info};
use futures_core::Stream;
use futures_util::StreamExt;

use crate::{events::ServerEvent, net::ExponentialBackoff};

#[derive(Debug, Clone)]
enum ConnectError {
    WrongStatusCode(StatusCode),
    WrongContentType(String),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match self {
           ConnectError::WrongStatusCode(code) => write!(f, "expecting status code 200 OK, found {} instead", code),
           ConnectError::WrongContentType(content_type) => write!(f, "expecting content-type text/event-stream, found '{}' instead", content_type),
       }
    }
}

impl Error for ConnectError {
}

struct MessageState {
    buffer: Vec<u8>,
    lines: VecDeque<String>,
}

impl MessageState {
    pub fn default() -> Self {
        MessageState { buffer: Vec::new(), lines: VecDeque::new() }
    }
}

pub enum MessageResult {
    NoMessages,
    Messages(Vec<ServerEvent>),
    ResponseError
}

pub struct Connection {
    stream: Pin<Box<dyn Stream<Item=Result<Bytes, reqwest::Error>>>>,
    state: MessageState,
}

impl Connection {
    pub fn new(
        stream: Pin<Box<dyn Stream<Item=Result<Bytes, reqwest::Error>>>>,
    ) -> Self {
        Connection { stream, state: MessageState::default() }
    }

    async fn try_connect(
        url: &str, user_agent: &String
    ) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::Client::builder().read_timeout(Duration::from_secs(30)).build()?;

        let request = client.request(
            Method::GET, reqwest::Url::parse(&url)?
        ).header(
            "User-Agent", user_agent
        ).header(
            "Accept", "text/event-stream"
        );

        let response = request.send().await?;

        if response.status() != StatusCode::OK {
            warn!(
                "Request to www.nationstates.net returned status code {}", response.status().as_u16()
            );

            match response.error_for_status() {
                Ok(res) => {
                    return Err(Box::new(ConnectError::WrongStatusCode(res.status())));
                }
                Err(err) => return Err(Box::new(err)),
            }
        }

        let headers = response.headers();
        let content_type = headers.get("Content-Type").ok_or_else(|| {
            warn!("Request to www.nationstates.net returned no content-type");

            Box::new(ConnectError::WrongContentType("".into()))
        })?.to_str()?;

        if !content_type.contains("event-stream") {
            warn!(
                "Request to www.nationstates.net returned incompatible content-type '{}'", content_type
            );

            return Err(Box::new(ConnectError::WrongContentType(content_type.to_owned())));
        }

        let stream = response.bytes_stream().boxed();

        Ok(Self::new(stream))
    }

    pub async fn connect(
        url: &str,
        user_agent: &String, 
        backoff: &mut ExponentialBackoff<'_>
    ) -> Connection {
        loop {
            match Self::try_connect(url, user_agent).await {
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

    async fn read_raw_messages(&mut self)
        -> Result<Vec<String>, Box<dyn Error>> 
    {
        while let Some(item) = self.stream.next().await {
            let data = item?;

            for slice in data.split_inclusive(|b| *b == b'\n') {
                self.state.buffer.extend(slice);

                if slice.ends_with(b"\n") {
                    let line = String::from_utf8(std::mem::take(&mut self.state.buffer))?;
                    self.state.lines.push_back(line);
                }
            }

            if self.state.lines.contains(&"\n".into()) {
                break;
            }
        }

        let mut result: Vec<String> = Vec::new();

        if let Some(last_delimiter) = self.state.lines.iter().rposition(|x| x == "\n") {
            let mut complete = self.state.lines.split_off(last_delimiter + 1);
            std::mem::swap(&mut complete, &mut self.state.lines);

            let mut iter = complete.into_iter();
            loop {
                let chunk = iter.by_ref().take_while(|line| line != "\n").collect::<String>();
                if !chunk.is_empty() { result.push(chunk); }
                else { break; }
            }
        }

        Ok(result)
    }

    pub fn deserialize_message(message: &String) -> Option<ServerEvent> {
        for line in message.lines() {
            let (label, data) = match line.split_once(": ") {
                Some(v) => v,
                None => continue,
            };

            match label {
                "" if data == "connected" => {
                    info!("Connected to NationStates");
                    return None;
                },
                "data" => {
                    let event: ServerEvent = match serde_json::from_str(data) {
                        Ok(v) => v,
                        Err(err) => {
                            warn!("Server returned malformed event '{}': {}", data, err);
                            return None;
                        }
                    };

                    return Some(event);
                },
                _ => continue,
            }
        }

        None
    }

    pub async fn read_messages(&mut self) -> Result<MessageResult, Box<dyn Error>> {
        let raw_messages = match self.read_raw_messages().await {
            Ok(v) => Ok(v),
            Err(err) => {
                let e = match err.downcast::<reqwest::Error>() {
                    Ok(req_err) => {
                        if req_err.is_timeout() {
                            warn!("Read from NationStates timed out: {}", req_err);
                            return Ok(MessageResult::ResponseError);
                        }

                        if req_err.is_decode() {
                            warn!("Error decoding response from NationStates: {}", req_err);
                            return Ok(MessageResult::ResponseError);
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

        let messages: Vec<ServerEvent> = raw_messages.iter().map(Self::deserialize_message).flatten().collect();

        if messages.is_empty() {
            return Ok(MessageResult::NoMessages);
        }

        Ok(MessageResult::Messages(messages))
    }
}