#![warn(clippy::pedantic)]
#![feature(once_cell)]

pub use context::Context;
use futures::StreamExt;
use std::{env, sync::Arc};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
use twilight_gateway::{cluster::ShardScheme, Cluster, Event, Intents};
use twilight_http::Client as HttpClient;

mod commands;
mod config;
mod context;
mod events;
mod rate_limiter;

async fn handle_event(shard_id: u64, event: Event, ctx: Arc<Context>) -> Res<()> {
    let res = match event {
        Event::Ready(ready) => events::ready(shard_id, ctx, ready).await,
        Event::InteractionCreate(interaction) => {
            events::interaction_create(shard_id, ctx, interaction).await
        }

        _ => {
            // debug!("Unhandled event {event:?}");
            Ok(())
        }
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

    let scheme = ShardScheme::Auto;

    let (cluster, mut events) = Cluster::builder(token.clone(), Intents::empty())
        .shard_scheme(scheme)
        .build()
        .await?;
    let cluster = Arc::new(cluster);

    let cluster_spawn = Arc::clone(&cluster);

    info!("Starting cluster...");
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = Arc::new(HttpClient::new(token.clone()));

    let application_id = {
        let req = http.current_user_application().exec().await?;

        req.model().await?.id
    };

    let ctx = Arc::new(Context {
        cluster,
        http,
        application_id,
    });

    while let Some((shard_id, event)) = events.next().await {
        tokio::spawn(handle_event(shard_id, event, ctx.clone()));
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
