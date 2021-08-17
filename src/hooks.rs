use log::{info, warn};
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::hook, DispatchError};
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[hook]
pub async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    let channel = msg.channel_id;

    info!(
        "Executing command `{}` in channel `{}` (ID: {})",
        cmd_name,
        channel
            .name(&ctx)
            .await
            .unwrap_or_else(|| "NAME_NOT_FOUND".to_string()),
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

    if error.is_ok() {
        info!(
            "Executed command `{}` in `{}` (ID: {})",
            cmd_name,
            channel
                .name(&ctx)
                .await
                .unwrap_or_else(|| "NAME_NOT_FOUND".to_string()),
            channel
        );
    } else if let Err(e) = error {
        warn!(
            "Failed to execute command `{}` in `{}` (ID: {}): {}",
            cmd_name,
            channel
                .name(&ctx)
                .await
                .unwrap_or_else(|| "NAME_NOT_FOUND".to_string()),
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
                    &format!("Try this again in {} seconds.\n\n**Saucenao currently has a restrictive rate limit, but I will try to improve it in the future.**\n    -Lys", duration.as_secs()),
                )
                .await;

            match a {
                Ok(_) => {}
                Err(e) => log::warn!("An error occurred: {}", e),
            }
        }
    }
}
