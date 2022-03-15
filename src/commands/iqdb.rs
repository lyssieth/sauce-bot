use std::sync::Arc;

use crate::{
    config::Config,
    events::{Cmd, Command},
    Context, Res,
};
use async_trait::async_trait;
use sauce_api::prelude::*;
use tracing::error;
use twilight_embed_builder::EmbedBuilder;
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::{
    channel::{embed::EmbedField, message::MessageFlags, Attachment},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;
use url::Url;

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
    async fn execute_link(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        let interaction_client = ctx.interaction_client();
        let link = self.link.clone().unwrap();

        if Url::parse(&link).is_err() {
            let resp = InteractionResponseDataBuilder::new()
                .content("Invalid link provided".to_owned())
                .flags(MessageFlags::EPHEMERAL);
            let resp = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(resp.build()),
            };

            interaction_client
                .create_response(command.interaction_id, &command.token, &resp)
                .exec()
                .await?;

            return Ok(());
        }

        self.execute_with_link(ctx, command, link).await
    }

    async fn execute_attachment(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        let interaction_client = ctx.interaction_client();
        let attachment = self.attachment.clone().unwrap();

        if let Some(content_type) = attachment.content_type {
            if !content_type.starts_with("image/") {
                let resp = InteractionResponseDataBuilder::new()
                    .content("Invalid attachment provided".to_owned())
                    .flags(MessageFlags::EPHEMERAL);
                let resp = InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(resp.build()),
                };

                interaction_client
                    .create_response(command.interaction_id, &command.token, &resp)
                    .exec()
                    .await?;

                return Ok(());
            }
        }

        self.execute_with_link(ctx, command, attachment.url).await
    }

    async fn execute_with_link(
        &self,
        ctx: Arc<Context>,
        command: Command,
        link: String,
    ) -> Res<()> {
        let interaction_client = ctx.interaction_client();

        let cfg = Config::load();
        let source = IQDB;
        let res = source.check_sauce(&link).await;

        if let Ok(result) = res {
            let mut embed =
                EmbedBuilder::new()
                    .title("Results")
                    .color(0x8B_D8C6)
                    .field(EmbedField {
                        name: "Original Link".to_owned(),
                        value: result.original_url.clone(),
                        inline: false,
                    });

            if result.items.is_empty() {
                embed = embed.field(EmbedField {
                    name: "Found zero results".to_owned(),
                    value: "Unable to find any results for the given link.".to_owned(),
                    inline: false,
                });
            } else {
                let mut i = cfg.settings().top_links() as usize;
                for x in result.items {
                    i -= 1;
                    embed = embed.field(EmbedField {
                        name: format!("Similarity: {:0.2}", x.similarity),
                        value: format!("**<{}>**", x.link),
                        inline: false,
                    });

                    if i == 0 {
                        break;
                    }
                }
            }

            let embed = embed.build()?;

            let mut resp = InteractionResponseDataBuilder::new().embeds(vec![embed]);
            if let Some(ephemeral) = self.ephemeral {
                if ephemeral {
                    resp = resp.flags(MessageFlags::EPHEMERAL);
                }
            }
            let resp = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(resp.build()),
            };

            interaction_client
                .create_response(command.interaction_id, &command.token, &resp)
                .exec()
                .await?;
        } else if let Err(e) = res {
            error!(?e, "Failed to execute");
            let mut resp = InteractionResponseDataBuilder::new()
                .content(format!("Failed to execute command: {}", e));
            if let Some(ephemeral) = self.ephemeral {
                if ephemeral {
                    resp = resp.flags(MessageFlags::EPHEMERAL);
                }
            }
            let resp = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(resp.build()),
            };

            interaction_client
                .create_response(command.interaction_id, &command.token, &resp)
                .exec()
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Cmd for Iqdb {
    async fn execute(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        let interaction_client = ctx.interaction_client();

        if self.link.is_none() && self.attachment.is_none() {
            let resp = InteractionResponseDataBuilder::new()
                .content("No image was provided, whether by link or attachment.".to_owned())
                .flags(MessageFlags::EPHEMERAL);
            let resp = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(resp.build()),
            };

            interaction_client
                .create_response(command.interaction_id, &command.token, &resp)
                .exec()
                .await?;

            return Ok(());
        }

        if self.link.is_some() {
            return self.execute_link(ctx, command).await;
        } else if self.attachment.is_some() {
            return self.execute_attachment(ctx, command).await;
        }

        Ok(())
    }
}
