use std::fmt;
use std::{collections::VecDeque, time::Duration};
use std::pin::Pin;

use bytes::Bytes;
use reqwest::{Method, StatusCode};
use log::warn;
use futures_core::Stream;
use futures_util::StreamExt;

use crate::config::InputConfig;

pub struct Connection {
    stream: Pin<Box<dyn Stream<Item=Result<Bytes, reqwest::Error>>>>,
}

impl Connection {
    pub fn new(
        stream: Pin<Box<dyn Stream<Item=Result<Bytes, reqwest::Error>>>>,
    ) -> Self {
        Connection { stream }
    }
}

pub struct ExponentialBackoff<'a> {
    index: usize,
    delays: &'a [u64],
}

impl ExponentialBackoff<'_> {
    pub fn new<'a>(
        delays: &'a [u64],
    ) -> ExponentialBackoff<'a> {
        assert!(!delays.is_empty(), "ExponentialBackoff must not be initialized with an empty array");
        ExponentialBackoff { index: 0, delays }
    }

    pub async fn wait(&mut self) {
        tokio::time::sleep(Duration::from_secs(self.delay())).await;
        self.index = self.index.saturating_add(1);
    }

    pub fn delay(&self) -> u64 {
        if self.index >= self.delays.len() {
            return *self.delays.last().expect("Delays should not be empty (verified by assertion)");
        }

        self.delays[self.index]
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}

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

impl std::error::Error for ConnectError {
}

pub async fn connect(config: &InputConfig, user_agent: &String) 
    -> Result<Connection, Box<dyn std::error::Error>> 
{
    let url = reqwest::Url::parse(&config.url)?;
    let client = reqwest::Client::builder().read_timeout(Duration::from_secs(30)).build()?;

    let request = client.request(
        Method::GET, url
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

    Ok(Connection::new(stream))
}

pub struct ReadMessageState {
    buffer: Vec<u8>,
    lines: VecDeque<String>,
}

impl ReadMessageState {
    pub fn default() -> Self {
        ReadMessageState { buffer: Vec::new(), lines: VecDeque::new() }
    }
}

pub async fn read_messages(conn: &mut Connection, state: &mut ReadMessageState) 
    -> Result<Vec<String>, Box<dyn std::error::Error>> 
{
    while let Some(item) = conn.stream.next().await {
        let data = item?;

        for slice in data.split_inclusive(|b| *b == b'\n') {
            state.buffer.extend(slice);

            if slice.ends_with(b"\n") {
                let line = String::from_utf8(std::mem::take(&mut state.buffer))?;
                state.lines.push_back(line);
            }
        }

        if state.lines.contains(&"\n".into()) {
            break;
        }
    }

    let mut result: Vec<String> = Vec::new();

    if let Some(last_delimiter) = state.lines.iter().rposition(|x| x == "\n") {
        let mut complete = state.lines.split_off(last_delimiter + 1);
        std::mem::swap(&mut complete, &mut state.lines);

        let mut iter = complete.into_iter();
        loop {
            let chunk = iter.by_ref().take_while(|line| line != "\n").collect::<String>();
            if !chunk.is_empty() { result.push(chunk); }
            else { break; }
        }
    }

    Ok(result)
}