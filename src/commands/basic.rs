use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};

#[group]
#[commands(ping)]
struct Basic;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn issue(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(
        &ctx,
        "To report an issue, please go to https://github.com/lyssieth/sauce-bot/issues",
    )
    .await?;

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}
