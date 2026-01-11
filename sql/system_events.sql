CREATE TABLE system_events (
    id BIGSERIAL PRIMARY KEY,
    time BIGINT NOT NULL,
    category TEXT NOT NULL,
    data TEXT[]
);