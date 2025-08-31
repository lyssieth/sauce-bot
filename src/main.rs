#![warn(clippy::pedantic, clippy::nursery, clippy::perf)]
#![deny(clippy::unwrap_used, clippy::panic)]

use sparkle_convenience::Bot;
use std::{env, sync::Arc};
use tokio::task::JoinSet;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, StreamExt as _};
use twilight_model::gateway::{
    payload::outgoing::UpdatePresence,
    presence::{Activity, ActivityType, MinimalActivity, Status},
};

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

    let (bot, shards) = Bot::new(
        token.clone(),
        Intents::empty(),
        EventTypeFlags::INTERACTION_CREATE | EventTypeFlags::READY,
    )
    .await?;
    let bot = Arc::new(bot);

    info!("Starting...");

    // let mut senders = Vec::with_capacity(shards.len());
    let mut tasks = Vec::with_capacity(shards.len());

    for shard in shards {
        // senders.push(shard.sender());
        tasks.push(tokio::spawn(runner(bot.clone(), shard)));
    }

    let joiners = JoinSet::from_iter(tasks);

    joiners.join_all().await;

    Ok(())
}

async fn runner(bot: Arc<Bot>, mut shard: Shard) {
    while let Some(event) = shard.next_event(bot.event_type_flags).await {
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                warn!(?error, "Error receiving event");

                continue;
            }
        };

        match event {
            Event::Ready(event) => {
                let activity = Activity::from(MinimalActivity {
                    kind: ActivityType::Playing,
                    name: "/help - slash commands!".to_string(),
                    url: None,
                });

                let command = UpdatePresence::new(vec![activity], false, None, Status::Online)
                    .expect("valid");

                shard.command(&command);

                info!(
                    "Shard {} ready, logged in as {}",
                    shard.id(),
                    event.user.name
                );

                let _ = events::ready(bot.clone()).await;
            }
            _ => {
                tokio::spawn(handle_event(event, bot.clone()));
            }
        }
    }
}

fn setup() -> Res<()> {
    if env::var("RUST_LIB_BACKTRACE").is_err() {
        unsafe {
            #[cfg(debug_assertions)]
            env::set_var("RUST_LIB_BACKTRACE", "1");
            #[cfg(not(debug_assertions))]
            env::set_var("RUST_LIB_BACKTRACE", "0");
        }
    }
    color_eyre::install()?;

    if env::var("RUST_LOG").is_err() {
        unsafe {
            #[cfg(debug_assertions)]
            env::set_var("RUST_LOG", "sauce_bot=debug");
            #[cfg(not(debug_assertions))]
            env::set_var("RUST_LOG", "sauce_bot=info");
        }
    }
    fmt::fmt()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
