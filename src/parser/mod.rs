mod patterns;
mod processors;

use regex::{Regex, RegexSet, Captures, Error};
use log::warn;
use std::collections::HashMap;

use crate::events::{ParsedEvent, ServerEvent};
use patterns::generate_patterns;
use processors::{Processor, generate_processor_map};

pub struct EventParser {
    pub patterns: Vec<(&'static str, Regex)>,
    pub regex_set: RegexSet,
    pub processors: HashMap<&'static str, Processor>,
}

impl EventParser {
    pub fn new() -> Result<Self, Error> {
        let (patterns, regex_set) = generate_patterns()?;

        Ok(Self {
            patterns,
            regex_set,
            processors: generate_processor_map()
        })
    }

    pub fn parse_server_event(&self, event: ServerEvent) -> Option<ParsedEvent> {
        let line = event.str.trim_end_matches('.');

        let Some((category, pattern)) = self.find_matching_regex(line) else {
            return Some(self.create_generic_event(&event, line, "unknown"));
        };

        if let Some(captures) = pattern.captures(line) {
            let regions = Self::extract_region_buckets(&event.buckets);

            let parsed_event = ParsedEvent::new(
                event.id.parse().unwrap_or(-1), 
                event.time,
                category
            );

            return self.process_regex_match(
                parsed_event,
                captures,
                &regions,
            ).or_else(|| {
                return Some(self.create_generic_event(&event, line, "skipped"));
            });
        }

        None
    }

    fn extract_region_buckets(buckets: &[String]) -> Vec<&str> {
        buckets.iter().filter_map(|b| {
            b.strip_prefix("region:")
        }).collect()
    }

    fn find_matching_regex(&self, line: &str) -> Option<&(&'static str, Regex)> {
        let matches = self.regex_set.matches(line);

        if !matches.matched_any() {
            warn!("Unmatched happening line: {}", line);
            return None;
        }

        // next().unwrap() should never fail as the !matched_any() case was already handled above
        Some(&self.patterns[matches.iter().next().unwrap()])
    }

    fn create_generic_event(&self, event: &ServerEvent, line: &str, category: &str) -> ParsedEvent {
        let mut event = ParsedEvent::new(
            event.id.parse().unwrap_or(-1), 
            event.time,
            category
        );

        event.data.push(line.to_owned());

        event
    }

    fn process_regex_match(
        &self,
        mut event: ParsedEvent,
        captures: Captures<'_>,
        regions: &[&str],
    ) -> Option<ParsedEvent> {
        let Some(processor) = self.processors.get(event.category.as_str()) else {
            if !event.category.contains("skip") {
                warn!(
                    "Happening {} matched category '{}' which doesn't have an associated processor", 
                    event.event, event.category
                );
            }

            return None;
        };

        processor.apply(&mut event, captures, regions);

        Some(event)
    }
}