FROM rustlang/rust:nightly as builder

LABEL name="lyssieth/sauce-bot"
LABEL maintainer="Lyssieth <lyssieth@rax.ee>"
ENV CONTAINER=true

RUN USER=root mkdir /config
RUN USER=root cargo new --bin sauce-bot
WORKDIR /sauce-bot

COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

RUN cargo build --release

RUN rm ./src/*.rs
RUN rm ./target/release/deps/sauce_bot*

COPY src ./src

RUN cargo build --release

FROM debian:bullseye-slim as runner

LABEL name="lyssieth/sauce-bot"
LABEL maintainer="Lyssieth <lyssieth@rax.ee>"

RUN apt update
RUN apt-get install --yes ca-certificates
RUN mkdir -p /config

COPY --from=builder /sauce-bot/target/release/sauce_bot /usr/bin/sauce-bot

VOLUME [ "/config" ]

ENTRYPOINT [ "sauce-bot" ]
