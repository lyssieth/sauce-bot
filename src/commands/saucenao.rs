use std::{lazy::SyncLazy, sync::Arc, time::Duration};

use crate::{
    config::Config,
    events::{Cmd, Command},
    Context, Res,
};
use async_trait::async_trait;
use r8limit::RateLimiter;
use sauce_api::prelude::*;
use tokio::sync::RwLock;
use tracing::error;
use twilight_embed_builder::EmbedBuilder;
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::{
    application::callback::InteractionResponse,
    channel::{embed::EmbedField, message::MessageFlags, Attachment},
};
use twilight_util::builder::CallbackDataBuilder;
use url::Url;

pub fn get() -> Vec<ApplicationCommandData> {
    vec![Saucenao::create_command()]
}

struct RateLimits {
    short_usage: RateLimiter,
    long_usage: RateLimiter,
}

impl RateLimits {
    pub fn limited(&mut self) -> bool {
        let a = self.short_usage.attempt();
        let b = self.long_usage.attempt();

        a || b
    }
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            short_usage: RateLimiter::new(6, Duration::from_secs(30)),
            long_usage: RateLimiter::new(200, Duration::from_secs(24 * 60 * 60)),
        }
    }
}

static mut RATE_LIMITS: SyncLazy<RwLock<RateLimits>> =
    SyncLazy::new(|| RwLock::new(RateLimits::default()));

#[derive(CreateCommand, CommandModel)]
#[command(
    name = "saucenao",
    desc = "Takes a link or attachment, and uses the saucenao backend to get results."
)]
pub struct Saucenao {
    /// The link to search for.
    link: Option<String>,

    /// An attachment to search for
    attachment: Option<Attachment>,
}

impl Saucenao {
    async fn execute_link(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        let interaction_client = ctx.interaction_client();
        let link = self.link.clone().unwrap();

        if Url::parse(&link).is_err() {
            let resp = CallbackDataBuilder::new()
                .content("Invalid link provided".to_owned())
                .flags(MessageFlags::EPHEMERAL);
            let resp = InteractionResponse::ChannelMessageWithSource(resp.build());

            interaction_client
                .interaction_callback(command.interaction_id, &command.token, &resp)
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
                let resp = CallbackDataBuilder::new()
                    .content("Invalid attachment provided".to_owned())
                    .flags(MessageFlags::EPHEMERAL);
                let resp = InteractionResponse::ChannelMessageWithSource(resp.build());

                interaction_client
                    .interaction_callback(command.interaction_id, &command.token, &resp)
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
        let mut source = SauceNao::new();
        source.set_api_key(cfg.credentials().saucenao_api_key().clone());
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

            let resp = CallbackDataBuilder::new()
                .embeds(vec![embed])
                .flags(MessageFlags::EPHEMERAL);
            let resp = InteractionResponse::ChannelMessageWithSource(resp.build());

            interaction_client
                .interaction_callback(command.interaction_id, &command.token, &resp)
                .exec()
                .await?;
        } else if let Err(e) = res {
            error!(?e, "Failed to execute");
            let resp = CallbackDataBuilder::new()
                .content(format!("Failed to execute command: {}", e))
                .flags(MessageFlags::EPHEMERAL);
            let resp = InteractionResponse::ChannelMessageWithSource(resp.build());

            interaction_client
                .interaction_callback(command.interaction_id, &command.token, &resp)
                .exec()
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Cmd for Saucenao {
    async fn execute(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        let mut rate_limits = unsafe { RATE_LIMITS.write().await };

        if rate_limits.limited() {
            return Ok(());
        }

        let interaction_client = ctx.interaction_client();

        if self.link.is_none() && self.attachment.is_none() {
            let resp = CallbackDataBuilder::new()
                .content("No image was provided, whether by link or attachment.".to_owned())
                .flags(MessageFlags::EPHEMERAL);
            let resp = InteractionResponse::ChannelMessageWithSource(resp.build());

            interaction_client
                .interaction_callback(command.interaction_id, &command.token, &resp)
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
