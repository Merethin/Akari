# Running Akari & Postgres via Docker

These instructions relate to running an instance of Akari that is attached to a Postgres database via the Docker Compose CLI. For instructions on running Akari from the Docker CLI, see the [README](https://github.com/merethin/akari/tree/main#setup).

## Project Structure

This guide assumes the following project directory structure, where `Akari` is a clone of this repository, and `pg-data` is an empty directory that will serve as a volume for the Postgres container.

```shell
.
├── .env
├── Akari
├── compose.yaml
├── config
│  └── akari.toml
└── pg-data
```

## Docker Compose

```
services:
  akari:
    build:
      context: ./Akari

    container_name: akari
    restart: unless-stopped

    depends_on:
      postgres:
        condition: service_healthy

    environment:
      NS_USER_AGENT: ${NS_USER_AGENT}
      DATABASE_URL: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB}

    volumes:
      - ./Akari:/app
      - ./config:/config

  postgres:
    image: postgres:16-alpine
    container_name: akari-postgres
    restart: unless-stopped

    environment:
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: ${POSTGRES_DB}

    ports:
      - "5432:5432"

    volumes:
      - ./pg-data:/var/lib/postgresql/data

    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5
```

## akari.toml

```
[input]
url = "https://www.nationstates.net/api/all"
workers = 2


[output.file]
enabled = false

[output.postgres]
enabled = true
skip_rmb_content = true
```

## .env

The following variables should be set in the .env file.

- NS_USER_AGENT
- POSTGRES_USER
- POSTGRES_PASSWORD
- POSTGRES_DB

## Start, Monitor, Stop

`docker compose up -d`

`docker compose logs`

`docker compose down`
