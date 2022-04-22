use crate::{config::Config, events::Command, Context, Res};
use color_eyre::eyre::eyre;
use num_traits::FromPrimitive;
use sauce_api::{Error, SauceResult};
use tracing::error;
use twilight_model::{
    channel::{embed::EmbedField, message::MessageFlags, Attachment},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};
use url::Url;

pub async fn get_link_from_link(ctx: &Context, command: &Command, link: String) -> Res<String> {
    let interaction_client = ctx.interaction_client();

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

        return Err(eyre!("invalid link provided"));
    }

    Ok(link)
}

pub async fn get_link_from_attachment(
    ctx: &Context,
    command: &Command,
    attachment: Attachment,
) -> Res<String> {
    let interaction_client = ctx.interaction_client();

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

            return Err(eyre!("invalid attachment provided"));
        }
    }

    Ok(attachment.url)
}

pub(crate) async fn respond(
    ctx: &Context,
    command: &Command,
    res: Result<SauceResult, Error>,
    cfg: Config,
    ephemeral: Option<bool>,
) -> Res<()> {
    let interaction_client = ctx.interaction_client();
    if let Ok(result) = res {
        let mut embed = EmbedBuilder::new()
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
            let mut items = result.items;

            items.sort_unstable_by_key(|c| i32::from_f32(c.similarity * 100f32.floor()));

            for x in items.iter().rev().take(cfg.settings().top_links() as usize) {
                embed = embed.field(EmbedField {
                    name: format!("Similarity: {:0.2}", x.similarity),
                    value: format!("**<{}>**", x.link),
                    inline: false,
                });
            }
        }

        let embed = embed.build();

        let mut resp = InteractionResponseDataBuilder::new().embeds(vec![embed]);
        if let Some(ephemeral) = ephemeral {
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
        if let Some(ephemeral) = ephemeral {
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

pub(crate) async fn respond_failure(ctx: &Context, command: &Command) -> Res<()> {
    let interaction_client = ctx.interaction_client();
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

    Ok(())
}

pub(crate) async fn get_link(
    ctx: &Context,
    command: &Command,
    link: &Option<String>,
    attachment: &Option<Attachment>,
) -> Res<String> {
    if link.is_some() {
        get_link_from_link(ctx, command, link.clone().unwrap()).await
    } else if attachment.is_some() {
        get_link_from_attachment(ctx, command, attachment.clone().unwrap()).await
    } else {
        Err(eyre!("fucked up"))
    }
}
