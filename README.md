# akari

Reliable, standalone SSE client and event parser for NationStates

## Introduction

Akari is, at its core, a program that listens to SSE events from NationStates, and tries its best to do so reliably, handling the necessary reconnections, with the goal of staying running 24/7 (even if NationStates itself has hiccups). It also parses said events into a more structured format.

Now, a program like this isn't very useful on its own - you can get slightly more advanced filtering than the NationStates Happenings view, but that's about it.

Akari is meant to be used as part of a distributed system - it is designed to freely broadcast events to other programs, which don't have to implement their own reconnection logic, can use a single combined SSE connection to the NS server and thus run less risk of running into the "5 concurrent connections" limit, and can freely benefit from the regex parsing and data structuring done by Akari.

## Inputs and Outputs

Akari takes events from an input source, parses them, and then passes them on to a number of outputs.

**Input**

Akari pulls events from https://www.nationstates.net/api/all by default. This can be edited in [akari.toml](config/akari.toml) to restrict the input feed to specific events, though this is not recommended as you can filter events directly from Akari.

Akari is multithreaded - it uses one thread to read SSE events from NS, one to broadcast parsed events to outputs, and a variable number of worker threads to parse the events and structure the data. The number of workers can be adjusted in the input section of [akari.toml](config/akari.toml). It is 2 by default, you probably won't need many more.

**Outputs**

Currently, there are 4 implemented output sources, each of which can be enabled or disabled separately and assigned an `include` list (to only broadcast certain events to that output) or an `exclude` list (to exclude certain events from being broadcast to that output).

- `console` - Prints events to stderr.
- `file` - Writes events to a log file.
- `redis` - Saves events to a Redis instance (using an indexed, searchable model).
- `rmq` - Broadcasts events to a RabbitMQ instance. Specifically, it broadcasts to a topic exchange (the exchange name can be configured in `akari.toml`). Applications can bind their queues to `*` or `#` to receive all events or bind to each category they want to listen to (categories are listed in [docs/happenings.md](docs/happenings.md)).

Events are always output in JSON format.

For an example of how to configure each of these outputs, check the default [akari.toml](config/akari.toml) configuration file.

## Event Parsing / Structured Events

A normal NationStates happening line looks like this:

`@@maxtopia@@ voted against the World Assembly Resolution "Condemn Testregionia"`

We know which rough category of events it belongs to, and that's about it. If you really want to, you can extract the additional information from the line.

With Akari, it's all done for you: 
```
{
    "event_id": 123123123,
    "timestamp": 246246246,
    "line": "@@maxtopia@@ voted against the World Assembly Resolution \"Condemn Testregionia\"",
    "actor": "maxtopia",
    "origin": "testregionia",
    "category": "wavote",
    "data": ["against", "Condemn Testregionia"]
}
```

Look at that "category" value. That's not the same as the NS "Vote" category - this category represents the unique event "voting on the current WA resolution". "Withdrawing a vote on the current WA resolution", also in the NS "Vote" category, has a separate category ID - "wrvote" in this case (short for "WA Add Vote" and "WA Remove Vote").

For a complete list of category IDs parsed by Akari as well as the regex patterns used to parse them, check [docs/happenings.md](docs/happenings.md).

There is more structured data. We can see the person who performed this action, the "actor", is extracted from the happening. The vote as well as the proposal name are in the "data" array. And the region this event originated in is stored in the "origin" field (this is not extracted from the happening line, but from a separate field provided by SSE).

## Setup

Run `cargo build --release` to compile the program. You'll need a recent version of Rust.

Run it with `NS_USER_AGENT=[YOUR MAIN NATION NAME] ./target/release/akari`.

Alternatively, you can set up a Docker container.

Building it: `docker build --tag akari .`

Running it: `docker run -e NS_USER_AGENT=[YOUR MAIN NATION NAME] akari`

Note: to pass your config file over to Akari, you must bind mount the directory it is in:

`docker run -e NS_USER_AGENT=[YOUR MAIN NATION NAME] -v ./config:/config akari`

Inside Docker, Akari looks for the config file in `/config/akari.toml`. If it isn't behaving like you expect, make sure the file is present/mounted in some way. The default Docker setup (without a bind mount) will just load the default configuration values.