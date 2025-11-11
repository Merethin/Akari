use regex::Captures;
use log::{info, warn};

use crate::{events::{Event, ServerEvent}, patterns::Happenings};

fn extract_region_buckets(buckets: &[String]) -> Vec<&str> {
    buckets.iter().filter_map(|b| {
        b.strip_prefix("region:")
    }).collect()
}

pub fn parse_message(message: &str) -> Option<ServerEvent> {
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

fn process_regex_match(
    mut event: Event,
    captures: Captures<'_>,
    regions: &[&str],
    happenings: &Happenings,
) -> Option<Event> {
    let Some(processor) = happenings.map.get(event.category.as_str()) else {
        warn!(
            "Happening {} matched category '{}' which doesn't have an associated processor", 
            event.event, event.category
        );

        return None;
    };

    processor.apply(&mut event, captures, regions);

    Some(event)
}

pub fn handle_server_message(
    evt: ServerEvent,
    happenings: &Happenings,
) -> Option<Event> {
    let line = evt.str.trim_end_matches('.');

    let matches = happenings.set.matches(line);

    if !matches.matched_any() {
        warn!("Unmatched happening line: {}", line);

        let mut event = Event::new(
            evt.id.parse().unwrap_or(-1), 
            evt.time,
            "unknown"
        );
        
        event.data.push(line.to_owned());
        return Some(event);
    }

    // next().unwrap() should never fail as the !matched_any() case was already handled above
    let (category, pattern) = &happenings.regexes[matches.iter().next().unwrap()];

    match pattern.captures(line) {
        Some(captures) => {
            let regions = extract_region_buckets(&evt.buckets);

            let event = Event::new(
                evt.id.parse().unwrap_or(-1), 
                evt.time,
                category
            );

            process_regex_match(
                event,
                captures,
                &regions,
                happenings,
            )
        },
        None => None,
    }
}