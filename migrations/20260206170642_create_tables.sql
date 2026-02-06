CREATE TABLE IF NOT EXISTS akari_events (
    event BIGINT PRIMARY KEY,
    time BIGINT NOT NULL,
    actor TEXT,
    receptor TEXT,
    origin TEXT,
    destination TEXT,
    category TEXT NOT NULL,
    data TEXT[]
);

CREATE TABLE IF NOT EXISTS akari_system_events (
    id BIGSERIAL PRIMARY KEY,
    time BIGINT NOT NULL,
    category TEXT NOT NULL,
    data TEXT[]
);