use std::sync::Arc;

use async_trait::async_trait;
use sauce_api::source::{self, Source};
use sparkle_convenience::Bot;
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::channel::Attachment;

use crate::{
    config::Config,
    events::{Cmd, Command},
    sauce_finder, Res,
};

pub fn get() -> Vec<ApplicationCommandData> {
    vec![FuzzySearch::create_command()]
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "fuzzysearch",
    desc = "Takes a link or attachment, and uses the fuzzysearch backend to get results."
)]
pub struct FuzzySearch {
    /// The link to search for.
    link: Option<String>,

    /// An attachment to search for
    attachment: Option<Attachment>,

    /// Whether the message should be hidden
    ephemeral: Option<bool>,
}

impl FuzzySearch {
    async fn execute_with_link(&self, bot: Arc<Bot>, command: Command, link: String) -> Res<()> {
        let cfg = Config::load();
        let source = source::fuzzysearch::FuzzySearch::create(
            cfg.credentials().fuzzysearch_api_key().clone(),
        )
        .await
        .expect("never fails");
        let res = source.check(&link).await;

        sauce_finder::respond(&bot, &command, res, cfg, self.ephemeral).await?;

        Ok(())
    }
}

#[async_trait]
impl Cmd for FuzzySearch {
    async fn execute(&self, bot: Arc<Bot>, command: Command) -> Res<()> {
        if self.link.is_none() && self.attachment.is_none() {
            sauce_finder::respond_failure(&bot, &command).await?;
        }

        let link = sauce_finder::get_link(&bot, &command, &self.link, &self.attachment).await?;

        self.execute_with_link(bot, command, link).await?;

        Ok(())
    }
}
