CREATE INDEX IF NOT EXISTS akari_events_time_event_idx ON akari_events (time, event);
DROP INDEX IF EXISTS akari_events_time_idx;