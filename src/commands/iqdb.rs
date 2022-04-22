use std::sync::Arc;

use crate::{
    config::Config,
    events::{Cmd, Command},
    sauce_finder, Context, Res,
};
use async_trait::async_trait;
use sauce_api::prelude::*;
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::channel::Attachment;

pub fn get() -> Vec<ApplicationCommandData> {
    vec![Iqdb::create_command()]
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "iqdb",
    desc = "Takes a link or attachment, and uses the iqdb backend to get results."
)]
pub struct Iqdb {
    /// The link to search for.
    link: Option<String>,

    /// An attachment to search for
    attachment: Option<Attachment>,

    /// Whether the message should be hidden
    ephemeral: Option<bool>,
}

impl Iqdb {
    async fn execute_with_link(
        &self,
        ctx: Arc<Context>,
        command: Command,
        link: String,
    ) -> Res<()> {
        let cfg = Config::load();
        let source = IQDB;
        let res = source.check_sauce(&link).await;

        sauce_finder::respond(&ctx, &command, res, cfg, self.ephemeral).await?;

        Ok(())
    }
}

#[async_trait]
impl Cmd for Iqdb {
    async fn execute(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        if self.link.is_none() && self.attachment.is_none() {
            sauce_finder::respond_failure(&ctx, &command).await?;
        }

        let link = sauce_finder::get_link(&ctx, &command, &self.link, &self.attachment).await?;

        self.execute_with_link(ctx, command, link).await?;

        Ok(())
    }
}
