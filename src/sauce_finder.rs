use crate::{config::Config, handle::Handle, Res};
use color_eyre::eyre::eyre;
use num_traits::FromPrimitive;
use sauce_api::{error::Error, source::Output};
use sparkle_convenience::reply::Reply;
use tracing::error;
use twilight_model::channel::{message::embed::EmbedField, Attachment};
use twilight_util::builder::embed::EmbedBuilder;
use url::Url;

pub async fn get_link_from_link(handle: &Handle, link: String) -> Res<String> {
    if Url::parse(&link).is_err() {
        handle
            .reply(Reply::new().ephemeral().content("Invalid link provided"))
            .await?;

        return Err(eyre!("invalid link provided"));
    }

    Ok(link)
}

pub async fn get_link_from_attachment(handle: &Handle, attachment: Attachment) -> Res<String> {
    if let Some(content_type) = attachment.content_type {
        if !content_type.starts_with("image/") {
            handle
                .reply(
                    Reply::new()
                        .ephemeral()
                        .content("Invalid attachment provided"),
                )
                .await?;

            return Err(eyre!("invalid attachment provided"));
        }
    }

    Ok(attachment.url)
}

pub async fn respond(
    handle: Handle,
    res: Result<Output, Error>,
    cfg: Config,
    ephemeral: Option<bool>,
) -> Res<()> {
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

            items.sort_unstable_by_key(|c| i32::from_f32(c.similarity * 100f32));

            for x in items.iter().rev().take(cfg.settings().top_links() as usize) {
                embed = embed.field(EmbedField {
                    name: format!("Similarity: {:0.2}", x.similarity),
                    value: format!("**<{}>**", x.link),
                    inline: false,
                });
            }
        }

        let embed = embed.build();

        let mut reply = Reply::new().embed(embed);

        if let Some(ephemeral) = ephemeral {
            if ephemeral {
                reply = reply.ephemeral();
            }
        }

        handle.reply(reply).await?;
    } else if let Err(e) = res {
        error!(?e, "Failed to execute");

        handle
            .reply(
                Reply::new()
                    .ephemeral()
                    .content(format!("Failed to execute command: {e}")),
            )
            .await?;
    }

    Ok(())
}

pub async fn respond_failure(handle: Handle) -> Res<()> {
    let reply = Reply::new()
        .ephemeral()
        .content("No image was provided, whether by link or attachment.");

    handle.reply(reply).await?;

    Ok(())
}

pub async fn get_link(
    handle: &Handle,
    link: &Option<String>,
    attachment: &Option<Attachment>,
) -> Res<String> {
    if let Some(link) = link {
        get_link_from_link(handle, link.clone()).await
    } else if let Some(attachment) = attachment {
        get_link_from_attachment(handle, attachment.clone()).await
    } else {
        Err(eyre!("fucked up"))
    }
}
