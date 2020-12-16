use eyre::eyre;
use log::info;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::StandardFramework,
    model::prelude::{Activity, Ready},
};

mod commands;
mod config;
mod hooks;
mod util;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        ctx.set_activity(Activity::playing("sauce!help")).await;
    }
}

type Result<T> = eyre::Result<T>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

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
        .before(hooks::before)
        .after(hooks::after)
        .group(&commands::BASIC_GROUP)
        .group(&commands::IQDB_GROUP)
        .group(&commands::SAUCENAO_GROUP)
        .group(&commands::ADMIN_GROUP);

    let mut client = Client::builder(cfg.credentials().token())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    info!("Starting bot...");
    if let Err(e) = client.start().await {
        Err(eyre!(
            "An error occurred while running the client: {:#?}",
            e
        ))
    } else {
        Ok(())
    }
}
