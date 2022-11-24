use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use crate::{
    config::Config,
    events::{Cmd, Command},
    rate_limiter::RateLimiter,
    sauce_finder, Context, Res,
};
use async_trait::async_trait;
use sauce_api::source::{saucenao::SauceNao, Source};
use tokio::sync::RwLock;
use twilight_interactions::command::{ApplicationCommandData, CommandModel, CreateCommand};
use twilight_model::{
    channel::{message::MessageFlags, Attachment},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub fn get() -> Vec<ApplicationCommandData> {
    vec![Saucenao::create_command()]
}

#[derive(Debug, Clone)]
struct RateLimits {
    short_usage: RateLimiter,
    long_usage: RateLimiter,
}

#[derive(Debug, Clone, Copy)]
enum Cause {
    Short,
    Long,
}

impl RateLimits {
    pub fn limited(&mut self) -> bool {
        let a = self.short_usage.attempt();
        let b = self.long_usage.attempt();

        !(a || b)
    }

    pub const fn cause(&self) -> Cause {
        if self.short_usage.remaining() == 0 {
            Cause::Long
        } else {
            Cause::Short
        }
    }

    pub fn new() -> Self {
        Self {
            short_usage: RateLimiter::new(6, Duration::from_secs(30)),
            long_usage: RateLimiter::new(200, Duration::from_secs(24 * 60 * 60)),
        }
    }
}

static mut RATE_LIMITS: LazyLock<Arc<RwLock<RateLimits>>> =
    LazyLock::new(|| Arc::new(RwLock::new(RateLimits::new())));

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

    /// Whether the message should be hidden
    ephemeral: Option<bool>,
}

impl Saucenao {
    async fn execute_with_link(
        &self,
        ctx: Arc<Context>,
        command: Command,
        link: String,
    ) -> Res<()> {
        let cfg = Config::load();
        let source = SauceNao::create(cfg.credentials().saucenao_api_key().clone())
            .await
            .expect("never fails");
        let res = source.check(&link).await;

        sauce_finder::respond(&ctx, &command, res, cfg, self.ephemeral).await?;

        Ok(())
    }
}

#[async_trait]
impl Cmd for Saucenao {
    async fn execute(&self, ctx: Arc<Context>, command: Command) -> Res<()> {
        let mut rate_limits = unsafe { RATE_LIMITS.write().await };

        let interaction_client = ctx.interaction_client();

        if rate_limits.limited() {
            let cause = rate_limits.cause();

            let resp = match cause {
                Cause::Short => InteractionResponseDataBuilder::new()
                    .content("You are being rate limited. Please wait up to 30 seconds before trying again. (sorry, the rate limits on SauceNao are like this. Consider `/support`ing the bot's creator)".to_owned())
                    .flags(MessageFlags::EPHEMERAL),
                Cause::Long => InteractionResponseDataBuilder::new()
                    .content("You are being rate limited. Please wait up to 24 hours for it to fix. (sorry, the rate limits on SauceNao are like this. Consider `/support`ing the bot's creator)".to_owned())
                    .flags(MessageFlags::EPHEMERAL),
            };

            let resp = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(resp.build()),
            };

            interaction_client
                .create_response(command.interaction_id, &command.token, &resp)
                .await?;

            return Ok(());
        }

        if self.link.is_none() && self.attachment.is_none() {
            sauce_finder::respond_failure(&ctx, &command).await?;
        }

        let link = sauce_finder::get_link(&ctx, &command, &self.link, &self.attachment).await?;

        self.execute_with_link(ctx, command, link).await?;

        Ok(())
    }
}
