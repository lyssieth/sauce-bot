use log::{info, warn};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
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
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    let channel = msg.channel_id;

    if error.is_ok() {
        info!(
            "Executed command `{}` in `{}` (ID: {})",
            cmd_name,
            channel
                .name(&ctx)
                .await
                .unwrap_or_else(|| "NAME_NOT_FOUND".to_string()),
            channel
        )
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
        )
    }
}
