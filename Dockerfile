FROM clux/muslrust:nightly-2023-05-10 as builder

LABEL name="lyssieth/sauce-bot"
LABEL maintainer="Lyssieth <lyssieth@rax.ee>"
ENV CONTAINER=true

RUN USER=root mkdir /config
RUN USER=root cargo new --bin sauce-bot
WORKDIR /sauce-bot

COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

# Cache index
RUN cargo update --dry-run

# Build dependencies
RUN cargo build --release

RUN rm ./src/*.rs
RUN rm ./target/release/deps/sauce_bot*

# Copy the actual source code lmao
COPY src ./src

# Build ourselves a release
RUN cargo build --release


FROM gcr.io/distroless/static as runner

LABEL name="lyssieth/sauce-bot"
LABEL maintainer="Lyssieth <lyssieth@rax.ee>"

COPY --from=builder /sauce-bot/target/release/sauce_bot /usr/bin/sauce-bot

VOLUME [ "/config" ]

ENTRYPOINT [ "sauce-bot" ]
