CREATE INDEX IF NOT EXISTS akari_events_category_event_idx ON akari_events (category, event);
DROP INDEX IF EXISTS akari_events_category_idx;