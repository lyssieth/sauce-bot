use eyre::eyre;
use serenity::{
    client::{Client, EventHandler},
    framework::StandardFramework,
};

mod commands;
mod config;

struct Handler;

impl EventHandler for Handler {}

type Result<T> = eyre::Result<T>;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::Config::load();
    let framework = StandardFramework::new()
        .configure(|c| {
            c.allow_dm(true)
                .by_space(true)
                .prefix(cfg.settings().prefix())
                .owners(cfg.settings().owner_ids_set())
                .no_dm_prefix(false)
                .ignore_bots(true)
                .ignore_webhooks(true)
                .case_insensitivity(true)
        })
        .group(&commands::BASIC_GROUP);

    let mut client = Client::builder(cfg.credentials().token())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        Err(eyre!(
            "An error occurred while running the client: {:#?}",
            e
        ))
    } else {
        Ok(())
    }
}
