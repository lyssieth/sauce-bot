use serenity::{
    client::Context,
    framework::standard::{
        CommandResult,
        macros::{command, group},
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
