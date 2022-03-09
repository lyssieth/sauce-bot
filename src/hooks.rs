use tracing::{info, warn};

pub async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    let channel = &msg
        .channel_id
        .to_channel(ctx)
        .await
        .expect("The channel doesn't exist????????");

    let name = if let Channel::Guild(channel) = channel {
        channel.name().to_owned()
    } else if let Channel::Private(channel) = channel {
        channel.name()
    } else {
        "NAME_NOT_FOUND".to_owned()
    };

    info!(
        "Executing command `{}` in channel `{}` (ID: {})",
        cmd_name,
        name.as_str(),
        channel
    );

    true
}

pub async fn after(
    ctx: &Context,
    msg: &Message,
    mut cmd_name: &str,
    error: Result<(), CommandError>,
) {
    let channel = &msg
        .channel_id
        .to_channel(ctx)
        .await
        .expect("The channel doesn't exist????????");

    let name = if let Channel::Guild(channel) = channel {
        channel.name().to_owned()
    } else if let Channel::Private(channel) = channel {
        channel.name()
    } else {
        "NAME_NOT_FOUND".to_owned()
    };

    if msg.content.contains("sauce!nao") || msg.content.contains("sauce!saucenao") {
        cmd_name = "saucenao:run";
    } else if msg.content.contains("sauce!iqdb") {
        cmd_name = "iqdb:run";
    }

    let m = msg
        .reply_ping(
            ctx,
            "**Warning: This bot is switching to slash commands soon. Please take care.**\n\nthis message will self destruct in 20 seconds",
        )
        .await
        .unwrap();

    let sp = ctx.clone();

    // hopefully set up the self destruct timer
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        m.delete(sp).await.unwrap();
    });

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

        msg.channel_id.send_message(&ctx, |m| m.content("Failed to execute command, due to an internal exception. Might be a good idea to run `sauce!issue`.".to_string())).await.unwrap();
    }
}
