use crate::config::Config;
use log::error;
use sauce_api::prelude::*;
use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};
use url::Url;

#[group()]
#[prefixes("saucenao", "nao")]
#[commands(run)]
#[default_command(run)]
struct Saucenao;

#[command]
#[min_args(0)]
#[max_args(1)]
#[bucket("saucenao-30s")]
#[bucket("saucenao-24h")]
async fn run(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = msg.channel_id;
    let link: Option<Url> = match args.single::<Url>() {
        Ok(url) => Some(url),
        Err(_) => None,
    };

    let num: Option<usize> = match args.single::<usize>() {
        Ok(amt) => Some(amt),
        Err(_) => None,
    };

    let link = match link {
        None => {
            if msg.attachments.is_empty() {
                None
            } else {
                let tmp = &msg.attachments[0];

                Some(Url::parse(&tmp.url)?)
            }
        }
        Some(a) => Some(a),
    };

    if link.is_none() {
        channel
            .send_message(&ctx, |m| {
                m.reference_message(msg)
                    .allowed_mentions(|a| a.empty_parse());
                m.content("No image was provided, whether by link or attachment.")
            })
            .await?;
        return Ok(());
    }
    let link = link.unwrap();

    let cfg = Config::load();
    let mut source = SauceNao::new();
    source.set_api_key(cfg.credentials().saucenao_api_key().clone());
    let res = source.check_sauce(link.as_ref()).await;

    if let Ok(result) = res {
        if cfg.settings().use_embeds() {
            channel
                .send_message(&ctx, |m| {
                    m.reference_message(msg)
                        .allowed_mentions(|a| a.empty_parse());
                    m.embed(|c| {
                        c.title("Results").color((139, 216, 198)).field(
                            "Original Link",
                            result.original_url,
                            false,
                        );

                        let mut i = if let Some(num) = num {
                            num
                        } else {
                            cfg.settings().top_links() as usize
                        };
                        if result.items.is_empty() {
                            c.field(
                                "Found zero results",
                                "Unable to find any results for the given link.",
                                false,
                            );
                        } else {
                            for x in result.items {
                                i -= 1;
                                c.field(
                                    format!("Similarity: {:0.2}", x.similarity),
                                    format!("**<{}>**", x.link),
                                    false,
                                );

                                if i == 0 {
                                    break;
                                }
                            }
                        }

                        c
                    })
                })
                .await?;
        } else {
            let mut lines = Vec::with_capacity(if let Some(num) = num {
                num
            } else {
                cfg.settings().top_links() as usize
            });

            let mut i = lines.capacity();
            for x in result.items {
                i -= 1;
                lines.push(format!("**{:0.2}%** - **<{}>**", x.similarity, x.link));

                if i == 0 {
                    break;
                }
            }

            let content = {
                let mut a = String::with_capacity(2000);
                if lines.is_empty() {
                    a.push_str("Found zero results.")
                } else {
                    for x in lines {
                        a.push_str(&x);
                        a.push('\n');
                    }
                }
                a
            };

            channel
                .send_message(&ctx, |m| {
                    m.reference_message(msg)
                        .allowed_mentions(|a| a.empty_parse())
                        .content(content)
                })
                .await?;
        }
    } else if let Err(e) = res {
        channel
            .send_message(&ctx, |m| {
                m.reference_message(msg)
                    .allowed_mentions(|a| a.empty_parse());
                m.content(format!("Failed to execute command: {}", e))
            })
            .await?;
        error!("Failed to execute: {}", e);
    }

    Ok(())
}
