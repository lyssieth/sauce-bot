use sauce_api::prelude::*;
use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};

use crate::config::Config;

#[group()]
#[prefix("saucenao")]
#[commands(run)]
#[default_command(run)]
struct Saucenao;

#[command]
#[num_args(1)]
async fn run(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let link = args.single::<String>()?;
    let chan = msg.channel_id;

    let cfg = Config::load();
    let mut source = SauceNao::new();
    source.set_api_key(cfg.credentials().saucenao_api_key().clone());
    let res = source.check_sauce(link).await;

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
                for x in lines {
                    a.push_str(&x);
                    a.push('\n');
                }
                a
            };

            chan.send_message(&ctx, |m| m.content(content)).await?;
        }
    } else if let Err(e) = res {
        chan.send_message(&ctx, |m| m.content(format!("Unable to get sauce: {}", e)))
            .await?;
    }

    Ok(())
}
