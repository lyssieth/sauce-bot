use crate::config::Config;
use log::error;
use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::channel::Message,
    prelude::Mentionable,
};

#[group]
#[commands(issue, help)]
struct Basic;

#[command]
async fn issue(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(
        &ctx,
        "To report an issue, please go to <https://github.com/lyssieth/sauce-bot/issues>",
    )
    .await?;

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let channel = msg.channel_id;
    let cfg = Config::load();
    let cfg = cfg.settings();

    match channel.send_message(&ctx, |m|
        m.reference_message(msg)
            .allowed_mentions(|a| a.empty_parse()).embed(|e|
                e.title("Help")
                    .description(format!("All commands use the `sauce!` prefix. Some commands might take a few seconds due to calling a potentially slow web service.\nSettings:\n- Links Displayed: {}\n- Using Embeds: {}", cfg.top_links(), cfg.use_embeds()))
                    .field("sauce!saucenao <link>", "Takes a link and uses the saucenao backend to get results. Fast, but has rate limits. Checks more locations.\nAlso works with `sauce!nao`\n\nRate limits:\n- 6 searches in 30 seconds\n- 200 searches in 24 hours\nThese apply globally across the bot.", false)
                    .field("sauce!iqdb <link>", "Takes a link and uses the iqdb backend to get results. Slower, without any rate limits, checks more locations.", false)
                    .field("sauce!issue", "Provides a link to the github to report issues with the bot.", false)
                    .field("sauce!help", "Provides help about the bot.", false)
                    .color((139, 216, 198))
            )
        ).await {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to send help: {}", err);
                if let Some(channel) = channel.to_channel(&ctx).await?.guild() {
                    msg.author.direct_message(&ctx, |m| {
                        m.embed(|e| {
                            e.title("Error")
                                .description("I failed to send the help message in the channel you requested. Information will be provided below.")
                                .field("Channel", format!("{}", channel.mention()), false)
                                .field("Guild", format!("{}", channel.guild_id), false)
                                .field("Error", format!("{}", err), false)
                        })
                    }).await?;
                }
            }
        }

    Ok(())
}
