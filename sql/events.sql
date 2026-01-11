CREATE TABLE events (
    event BIGINT PRIMARY KEY,
    time BIGINT NOT NULL,
    actor TEXT,
    receptor TEXT,
    origin TEXT,
    destination TEXT,
    category TEXT NOT NULL,
    data TEXT[]
);

ALTER TABLE events ALTER COLUMN data TYPE jsonb USING to_jsonb(data);