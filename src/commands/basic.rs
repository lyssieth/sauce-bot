use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group}, CommandResult,
    },
    model::channel::Message,
};

use crate::config::Config;

#[group]
#[commands(issue, help)]
struct Basic;

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
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let chan = msg.channel_id;
    let cfg = Config::load();
    let cfg = cfg.settings();

    chan.send_message(&ctx, |m| 
            m.embed(|e| 
                e.title("Help")
                    .description(format!("All commands use the `sauce!` prefix. Some commands might take a few seconds due to calling a potentially slow web service.\nSettings:\n- Links Displayed: {}\n- Using Embeds: {}", cfg.top_links(), cfg.use_embeds()))
                    .field("sauce!saucenao <link>", "Takes a link and uses the saucenao backend to get results. Best option currently.", false)
                    .field("sauce!iqdb <link>", "Takes a link and uses the iqdb backend to get results. Slower than saucenao, with less locations checked.", false)
                    .field("sauce!issue", "Provides a link to the github to report issues with the bot.", false)
                    .field("sauce!help", "Provides help about the bot.", false)
                    .color((22, 184, 184))
            )
        ).await?;

    Ok(())
}
