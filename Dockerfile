
FROM rust:1.90-alpine AS build

WORKDIR /usr/src/akari
RUN cargo init --bin .

RUN apk add libressl-dev musl-dev

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./migrations ./migrations

RUN rm ./target/release/deps/akari*
RUN cargo build --release

FROM rust:1.90-alpine

WORKDIR /

COPY --from=build /usr/src/akari/target/release/akari /usr/local/bin/akari

CMD ["akari"]