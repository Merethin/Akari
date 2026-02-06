CREATE INDEX IF NOT EXISTS akari_events_time_idx ON akari_events (time);
CREATE INDEX IF NOT EXISTS akari_events_actor_idx ON akari_events (actor);
CREATE INDEX IF NOT EXISTS akari_events_receptor_idx ON akari_events (receptor);
CREATE INDEX IF NOT EXISTS akari_events_origin_idx ON akari_events (origin);
CREATE INDEX IF NOT EXISTS akari_events_destination_idx ON akari_events (destination);
CREATE INDEX IF NOT EXISTS akari_events_category_idx ON akari_events (category);