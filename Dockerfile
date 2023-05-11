FROM clux/muslrust:nightly-2023-05-10 as builder

LABEL name="lyssieth/sauce-bot"
LABEL maintainer="Lyssieth <lyssieth@rax.ee>"
ENV CONTAINER=true

RUN USER=root mkdir /config
RUN USER=root cargo new --bin sauce-bot
WORKDIR /sauce-bot


COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

# Make a fake source file so we can cache dependencies
RUN mkdir -p ./src
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > ./src/main.rs

# Cache index
RUN cargo fetch

# Build & cache dependencies
RUN cargo build --release

RUN rm ./src/*.rs
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/sauce_bot*

# Copy the actual source code lmao
COPY src ./src

# Build ourselves a release
RUN cargo build --release


FROM gcr.io/distroless/static as runner

LABEL name="lyssieth/sauce-bot"
LABEL maintainer="Lyssieth <lyssieth@rax.ee>"

COPY --from=builder /sauce-bot/target/x86_64-unknown-linux-musl/release/sauce_bot /usr/bin/sauce-bot

VOLUME [ "/config" ]

ENTRYPOINT [ "sauce-bot" ]
