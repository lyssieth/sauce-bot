#![warn(clippy::pedantic, clippy::nursery, clippy::perf)]
#![deny(clippy::unwrap_used, clippy::panic)]
#![feature(lazy_cell)]

use futures::StreamExt;
use sparkle_convenience::Bot;
use std::{env, process::exit, sync::Arc};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use twilight_gateway::{stream::ShardEventStream, Event, EventTypeFlags, Intents};

mod commands;
mod config;
mod events;
mod handle;
mod rate_limiter;
mod sauce_finder;

async fn handle_event(event: Event, bot: Arc<Bot>) -> Res<()> {
    let res = match event {
        Event::InteractionCreate(interaction) => events::interaction_create(bot, interaction).await,

        _ => Ok(()),
    };

    if let Err(e) = res {
        error!("Error handling event: {e}");
    }

    Ok(())
}

type Res<T> = color_eyre::Result<T>;

#[tokio::main]
async fn main() -> Res<()> {
    setup()?;

    let cfg = config::Config::load();
    let token = cfg.credentials().token();

    let (bot, mut shards) = Bot::new(
        token.clone(),
        Intents::empty(),
        EventTypeFlags::INTERACTION_CREATE | EventTypeFlags::READY,
    )
    .await?;
    let bot = Arc::new(bot);

    info!("Starting...");

    let mut stream = ShardEventStream::new(shards.0.iter_mut());

    while let Some((shard, event)) = stream.next().await {
        let event = match event {
            Ok(event) => event,
            Err(source) => {
                if source.is_fatal() {
                    error!(?source, "Fatal error receiving event");

                    exit(1);
                }

                warn!(?source, "Error receiving event");

                continue;
            }
        };

        match event {
            Event::Ready(event) => events::ready(shard, bot.clone(), event).await?,
            _ => {
                tokio::spawn(handle_event(event, bot.clone()));
            }
        }
    }

    Ok(())
}

fn setup() -> Res<()> {
    if env::var("RUST_LIB_BACKTRACE").is_err() {
        #[cfg(debug_assertions)]
        env::set_var("RUST_LIB_BACKTRACE", "1");
        #[cfg(not(debug_assertions))]
        env::set_var("RUST_LIB_BACKTRACE", "0");
    }
    color_eyre::install()?;

    if env::var("RUST_LOG").is_err() {
        #[cfg(debug_assertions)]
        env::set_var("RUST_LOG", "sauce_bot=debug");
        #[cfg(not(debug_assertions))]
        env::set_var("RUST_LOG", "sauce_bot=info");
    }
    fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
