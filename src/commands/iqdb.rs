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

use crate::config::Config;

#[group()]
#[prefix("iqdb")]
#[commands(run)]
#[default_command(run)]
struct Iqdb;

#[command]
#[num_args(1)]
async fn run(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let chan = msg.channel_id;
    let link: Option<Url> = match args.single::<Url>() {
        Ok(url) => Some(url),
        Err(_) => {
            chan.send_message(&ctx, |m| {
                m.content("Sorry, but you provided an invalid URL.")
            })
            .await?;

            None
        }
    };

    if link.is_none() {
        return Ok(());
    }
    let link = link.unwrap();

    let cfg = Config::load();
    let source = IQDB;
    let res = source.check_sauce(link.to_string()).await;

    if let Ok(result) = res {
        if cfg.settings().use_embeds() {
            chan.send_message(&ctx, |m| {
                m.embed(|c| {
                    c.title("Results").color((139, 216, 198)).field(
                        "Original Link",
                        result.original_url,
                        false,
                    );

                    let mut i = cfg.settings().top_links();
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
            let mut lines = Vec::new();

            let mut i = cfg.settings().top_links();
            for x in result.items {
                i -= 1;
                lines.push(format!("**{:0.2}%** - **<{}>**", x.similarity, x.link));

                if i == 0 {
                    break;
                }
            }

            let content = {
                let mut a = String::new();
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

            chan.send_message(&ctx, |m| m.content(content)).await?;
        }
    } else if let Err(e) = res {
        chan.send_message(&ctx, |m| {
            m.content(format!("Failed to execute command: {}", e))
        })
        .await?;
    }

    Ok(())
}
