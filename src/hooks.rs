use tracing::{info, warn};
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::hook, DispatchError};
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[hook]
pub async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    let channel = msg.channel_id;

    let name = channel
        .name(&ctx)
        .await
        .unwrap_or_else(|| "NAME_NOT_FOUND".to_string());

    info!(
        "Executing command `{}` in channel `{}` (ID: {})",
        cmd_name,
        name.as_str(),
        channel
    );

    true
}

#[hook]
pub async fn after(
    ctx: &Context,
    msg: &Message,
    mut cmd_name: &str,
    error: Result<(), CommandError>,
) {
    let channel = msg.channel_id;

    if msg.content.contains("sauce!nao") || msg.content.contains("sauce!saucenao") {
        cmd_name = "saucenao:run";
    } else if msg.content.contains("sauce!iqdb") {
        cmd_name = "iqdb:run";
    }

    let name = channel
        .name(&ctx)
        .await
        .unwrap_or_else(|| "NAME_NOT_FOUND".to_string());

    if error.is_ok() {
        info!(
            "Executed command `{}` in `{}` (ID: {})",
            cmd_name,
            name.as_str(),
            channel
        );
    } else if let Err(e) = error {
        warn!(
            "Failed to execute command `{}` in `{}` (ID: {}): {}",
            cmd_name,
            name.as_str(),
            channel,
            e
        );

        channel.send_message(&ctx, |m| m.content("Failed to execute command, due to an internal exception. Might be a good idea to run `sauce!issue`.".to_string())).await.unwrap();
    }
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(duration) = error {
        if msg.content.contains("sauce!saucenao") {
            let a = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.\n\n**Saucenao has a restrictive time limit, so if you want it raised, please support my patreon. `sauce!support`**\n    - Lys", duration.as_secs()),
                )
                .await;

            match a {
                Ok(_) => {}
                Err(e) => tracing::warn!("An error occurred: {}", e),
            }
        }
    }
}
