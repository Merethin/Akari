use serde::{Deserialize, Serialize};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{SystemTime, UNIX_EPOCH}};

#[derive(Deserialize, Debug)]
pub struct ServerEvent {
    pub id: String,
    pub time: u64,
    pub str: String,
    pub buckets: Vec<String>,
    // pub htmlStr: String
    // pub rmbMessage: Option<String>
}

#[derive(Debug)]
pub struct SystemEvent {
    pub time: u64,
    pub category: &'static str,
    pub data: Vec<String>
}

fn now_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect(
        "Current system time should be later than the Unix epoch"
    ).as_secs()
}

impl SystemEvent {
    pub fn connection_initialized() -> SequencedEvent {
        SequencedEvent::wrap_system(SystemEvent {
            time: now_timestamp(),
            category: "conninit",
            data: vec![]
        })
    }

    pub fn connection_dropped(last_event_id: i64) -> SequencedEvent {
        SequencedEvent::wrap_system(SystemEvent {
            time: now_timestamp(),
            category: "conndrop",
            data: vec![last_event_id.to_string()]
        })
    }

    pub fn events_missed(
        events_missed: i64,
        last_event_id: i64,
        current_id: i64,
    ) -> SequencedEvent {
        SequencedEvent::wrap_system(SystemEvent {
            time: now_timestamp(),
            category: "connmiss",
            data: vec![events_missed.to_string(), last_event_id.to_string(), current_id.to_string()]
        })
    }
}

pub enum Message {
    Server(ServerEvent),
    System(SystemEvent),
}

static SEQUENCE_ID: AtomicUsize = AtomicUsize::new(0);

pub struct SequencedEvent {
    seq_id: usize,
    event: Message,
}

impl SequencedEvent {
    pub fn wrap_server(event: ServerEvent) -> Self {
        Self {
            seq_id: SEQUENCE_ID.fetch_add(1, Ordering::Relaxed),
            event: Message::Server(event)
        }
    }

    pub fn wrap_system(event: SystemEvent) -> Self {
        Self {
            seq_id: SEQUENCE_ID.fetch_add(1, Ordering::Relaxed),
            event: Message::System(event)
        }
    }

    pub fn get_event(self) -> Message {
        self.event
    }

    pub fn sequence_id(&self) -> usize {
        self.seq_id
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ParsedEvent {
    pub event: i64,
    pub time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receptor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    pub category: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<String>,
}

impl ParsedEvent {
    pub fn new(
        event_id: i64,
        time: u64,
        category: &str,
    ) -> Self {
        ParsedEvent {
            event: event_id, 
            time,
            actor: None, 
            receptor: None, 
            origin: None, 
            destination: None, 
            category: category.to_owned(), 
            data: Vec::new() 
        }
    }
}