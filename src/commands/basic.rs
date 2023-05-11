use std::sync::Arc;

use async_trait::async_trait;
use sparkle_convenience::{reply::Reply, Bot};
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::channel::message::embed::EmbedField;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    config::Config,
    events::{Cmd, Command},
    Res,
};

pub fn get() -> Vec<ApplicationCommandData> {
    let help = HelpCommand::create_command();

    let issue = IssueCommand::create_command();
    let support = SupportCommand::create_command();

    vec![help, issue, support]
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "help", desc = "Provides help for SauceBot")]
pub struct HelpCommand;

#[async_trait]
impl Cmd for HelpCommand {
    async fn execute(&self, bot: Arc<Bot>, command: Command) -> Res<()> {
        let handle = bot.interaction_handle(&command);

        let cfg = Config::load();
        let cfg = cfg.settings();

        let embed = EmbedBuilder::new()
            .title("Help")
            .description(format!("All commands use the `sauce!` prefix. Some commands might take a few seconds due to calling a potentially slow web service.\nSettings:\n- Links Displayed: {}", cfg.top_links()))
            .field(EmbedField {
                name: "sauce!saucenao <link>".to_owned(),
                value: "Takes a link and uses the saucenao backend to get results. Fast, but has rate limits. Checks more locations.\nAlso works with `sauce!nao`\n\nRate limits:\n- 6 searches in 30 seconds\n- 200 searches in 24 hours\nThese apply globally across the bot.".to_owned(),
                inline: false,
            })
            .field(EmbedField {
                name: "sauce!iqdb <link>".to_owned(),
                value: "Takes a link and uses the iqdb backend to get results. Slower, without any rate limits, checks more locations.".to_owned(),
                inline: false,
            })
            .field(EmbedField {
                name: "sauce!fuzzy <link>".to_owned(),
                value: "Takes a link and uses the fuzzysearch backend to get results. Fast, has rate limits\nBetter for furry/brony art than anything else.".to_owned(),
                inline: false,
            })
            .field(EmbedField {
                name: "sauce!issue".to_owned(),
                value: "Provides a link to the github to report issues with the bot.".to_owned(),
                inline: false,
            })
            .field(EmbedField {
                name: "sauce!support".to_owned(),
                value: "Provides a link to ways to support the bot.\nThis will help keep the VPS up and running, as well as increase SauceNao rate limits.".to_owned(),
                inline: false,
            })
            .field(EmbedField {
                name: "sauce!help".to_owned(),
                value: "Provides help about the bot.".to_owned(),
                inline: false,
            })
            .color(0x8B_D8C6)
            .build();

        handle.reply(Reply::new().embed(embed).ephemeral()).await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "issue",
    desc = "Provides information about how to report an issue"
)]
pub struct IssueCommand;

#[async_trait]
impl Cmd for IssueCommand {
    async fn execute(&self, bot: Arc<Bot>, command: Command) -> Res<()> {
        let handle = bot.interaction_handle(&command);

        handle.reply(Reply::new().content("To report an issue, please go to <https://github.com/lyssieth/sauce-bot/issues>").ephemeral()).await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "support",
    desc = "Provides information about how to support the continued work of SauceBot"
)]
pub struct SupportCommand;

#[async_trait]
impl Cmd for SupportCommand {
    async fn execute(&self, bot: Arc<Bot>, command: Command) -> Res<()> {
        let handle = bot.interaction_handle(&command);

        let embed = EmbedBuilder::new()
            .title("Support")
                    .description("All the ways to support SauceBot.\n\nAny money gained through this will first go towards the VPS and SauceNao rate limits, after which it will go into my pocket.")
                    .field(EmbedField {
                        name: "Patreon".to_owned(),
                        value: "Monthly only. [Link](https://patreon.com/lyssieth)".to_owned(),
                        inline: false,
                    })
                    .field(EmbedField {
                        name: "Github Sponsor".to_owned(),
                        value: "Both one-time and monthly. [Link](https://github.com/sponsors/lyssieth)".to_owned(),
                        inline: false
                    })
            .color(0x8B_D8C6)
            .build();

        handle.reply(Reply::new().embed(embed).ephemeral()).await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "invite", desc = "Provides an invite link for the bot")]
pub struct InviteCommand;

#[async_trait]
impl Cmd for InviteCommand {
    async fn execute(&self, bot: Arc<Bot>, command: Command) -> Res<()> {
        let handle = bot.interaction_handle(&command);

        handle.reply(Reply::new().content("To invite the bot to your server, please go to <https://discord.com/oauth2/authorize?client_id=778822593293058051&scope=bot,applications.commands&permissions=19456>").ephemeral()).await?;

        Ok(())
    }
}
