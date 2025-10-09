use redis_om::JsonModel;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ServerEvent {
    pub id: String,
    pub time: u64,
    pub str: String,
    pub buckets: Vec<String>,
    // pub htmlStr: String
}

#[derive(Debug)]
pub struct SystemEvent {
    pub time: u64,
    pub line: String,
    pub category: &'static str,
    pub data: Vec<String>
}

impl SystemEvent {
    pub fn connection_initialized(time: u64) -> Self {
        SystemEvent {
            time,
            line: "System: Connection to NationStates initialized / resumed".into(),
            category: "conninit",
            data: vec![]
        }
    }

    pub fn connection_dropped(time: u64) -> Self {
        SystemEvent {
            time,
            line: "System: Connection to NationStates lost".into(),
            category: "conndrop",
            data: vec![]
        }
    }

    pub fn events_missed(
        time: u64,
        events_missed: i64,
        last_event_id: i64,
        current_id: i64,
    ) -> Self {
        SystemEvent {
            time,
            line: format!(
                "System: {} NationStates events missed (from {} to {})", 
                events_missed, last_event_id, current_id
            ),
            category: "connmiss",
            data: vec![events_missed.to_string(), last_event_id.to_string(), current_id.to_string()]
        }
    }
}

pub enum Message {
    Server(ServerEvent),
    System(SystemEvent),
}

#[derive(JsonModel, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Event {
    #[serde(skip_serializing_if = "String::is_empty")]
    id: String,
    event_id: i64,
    time: u64,
    pub line: String,
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

impl Event {
    pub fn new(
        event_id: i64,
        time: u64,
        line: &str,
        category: &str,
    ) -> Self {
        Event { 
            id: "".into(), 
            event_id, 
            time,
            line: line.to_owned(), 
            actor: None, 
            receptor: None, 
            origin: None, 
            destination: None, 
            category: category.to_owned(), 
            data: Vec::new() 
        }
    }
}