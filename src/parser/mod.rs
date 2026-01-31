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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_region_buckets() {
        let buckets = &["all".to_string()];
        let regions = EventParser::extract_region_buckets(buckets);
        assert_eq!(regions, Vec::<&str>::new());

        let buckets = &["all".to_string(), "endo".to_string(), "region:testregionia".to_string()];
        let regions = EventParser::extract_region_buckets(buckets);
        assert_eq!(regions, vec!["testregionia"]);

        let buckets = &["region:the_pacific".to_string(), "change".to_string(), "region:testregionia".to_string()];
        let regions = EventParser::extract_region_buckets(buckets);
        assert_eq!(regions, vec!["the_pacific", "testregionia"]);
    }

    #[test]
    fn test_find_matching_regex() {
        let parser = EventParser::new().unwrap();

        let (category, pattern) = parser.find_matching_regex("@@a@@ became WA Delegate of %%b%%").unwrap();
        assert_eq!(*category, "ndel");
        assert_eq!(pattern.as_str(), "^@@([0-9a-z_-]+)@@ became WA Delegate of %%([0-9a-z_-]+)%%$");

        let (category, pattern) = parser.find_matching_regex(
            "@@a@@ changed its national b to \"c\", its d to \"e\" and its f to \"g\""
        ).unwrap();

        assert_eq!(*category, "chfield");
        assert_eq!(
            pattern.as_str(), 
            r#"^@@([0-9a-z_-]+)@@ changed its national ([a-z ]+) to "([^"]*)"((?:,? (?:and )?its [a-z ]+ to "[^"]*")+)?$"#
        );
    }

    #[test]
    fn test_simple_parse_server_event() {
        let parser = EventParser::new().unwrap();

        let event = parser.parse_server_event(ServerEvent {
            id: "100".to_string(),
            time: 200,
            str: "@@a@@ changed a custom banner.".to_string(),
            buckets: vec!["region:b".to_string()]
        });

        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.event, 100);
        assert_eq!(event.time, 200);
        assert_eq!(event.actor, Some("a".to_string()));
        assert!(event.receptor.is_none());
        assert_eq!(event.origin, Some("b".to_string()));
        assert!(event.destination.is_none());
        assert_eq!(&event.category, "chbanner");
        assert!(event.data.is_empty());
    }

    #[test]
    fn test_complex_parse_server_event() {
        let parser = EventParser::new().unwrap();

        let event = parser.parse_server_event(ServerEvent {
            id: "100".to_string(),
            time: 200,
            str: r#"@@a@@ granted <i class="b"></i>Bb and <i class="c"></i>Cc authority and removed <i class="e"></i>Ex authority from @@d@@ and renamed the office from "l" to "s" in %%m%%."#.to_string(),
            buckets: vec!["region:b".to_string()]
        });

        assert!(event.is_some());

        let event = event.unwrap();
        assert_eq!(event.event, 100);
        assert_eq!(event.time, 200);
        assert_eq!(event.actor, Some("a".to_string()));
        assert_eq!(event.receptor, Some("d".to_string()));
        assert_eq!(event.origin, Some("m".to_string())); // region in pattern overrides bucket
        assert!(event.destination.is_none());
        assert_eq!(&event.category, "rochname");
        assert_eq!(event.data, vec!["l", "s", "+BC", "-X"]);
    }
}