use std::sync::Arc;

use async_trait::async_trait;
use sauce_api::source::{self, Source};
use sparkle_convenience::{interaction::DeferVisibility, Bot};
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::channel::Attachment;

use crate::{
    config::Config,
    events::{Cmd, Command},
    handle::Handle,
    handle::SpecialHandler,
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
    async fn execute_with_link(&self, handle: Handle, link: String) -> Res<()> {
        let cfg = Config::load();
        let source = source::fuzzysearch::FuzzySearch::create(
            cfg.credentials().fuzzysearch_api_key().clone(),
        )
        .await
        .expect("never fails");
        let res = source.check(&link).await;

        sauce_finder::respond(handle, res, cfg, self.ephemeral).await?;

        Ok(())
    }
}

#[async_trait]
impl Cmd for FuzzySearch {
    async fn execute(&self, bot: Arc<Bot>, command: Command) -> Res<()> {
        let handle = bot.handle(&command.interaction);
        if self.link.is_none() && self.attachment.is_none() {
            sauce_finder::respond_failure(handle).await?;
            return Ok(());
        }

        handle
            .defer(if self.ephemeral.unwrap_or(false) {
                DeferVisibility::Ephemeral
            } else {
                DeferVisibility::Visible
            })
            .await?;

        let link = sauce_finder::get_link(&handle, &self.link, &self.attachment).await?;

        self.execute_with_link(handle, link).await?;

        Ok(())
    }
}
