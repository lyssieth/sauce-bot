use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};

#[group()]
#[prefix("admin")]
#[owners_only]
#[commands(name, avatar)]
struct Admin;

#[command]
#[min_args(0)]
#[max_args(1)]
async fn name(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut self_user = ctx.cache.current_user().await;
    let channel = msg.channel_id;

    if args.is_empty() {
        channel
            .send_message(&ctx, |m| {
                m.reference_message(msg)
                    .allowed_mentions(|a| a.empty_parse());
                m.content(format!("Current name is: `{}`", self_user.name))
            })
            .await?;
    } else {
        let name = args.single::<String>()?;

        if name.is_empty() || name.len() > 32 {
            channel
                .send_message(&ctx, |m| {
                    m.reference_message(msg)
                        .allowed_mentions(|a| a.empty_parse());
                    m.content(format!("Expected length of 1-32, got {}", name.len()))
                })
                .await?;
            return Ok(());
        }

        self_user.edit(&ctx, |e| e.username(&name)).await?;
        channel
            .send_message(&ctx, |m| {
                m.reference_message(msg)
                    .allowed_mentions(|a| a.empty_parse())
                    .content(format!("Set name to {}", name))
            })
            .await?;
    }

    Ok(())
}

#[command]
#[min_args(0)]
#[max_args(1)]
async fn avatar(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut self_user = ctx.cache.current_user().await;
    let channel = msg.channel_id;

    if args.is_empty() {
        if msg.attachments.is_empty() {
            channel
                .send_message(&ctx, |m| {
                    m.reference_message(msg)
                        .allowed_mentions(|a| a.empty_parse());
                    m.content(format!(
                        "Current avatar: {}",
                        self_user
                            .avatar_url()
                            .unwrap_or_else(|| { self_user.default_avatar_url() })
                    ))
                })
                .await?;
        } else {
            let new_avatar = &msg.attachments[0].url;

            self_user
                .edit(&ctx, |e| e.avatar(Some(new_avatar.as_str())))
                .await?;
            channel
                .send_message(&ctx, |m| {
                    m.reference_message(msg)
                        .allowed_mentions(|a| a.empty_parse())
                        .content(format!("Set avatar to {}", new_avatar))
                })
                .await?;
        }
    } else {
        let new_avatar = args.single::<String>()?;

        self_user
            .edit(&ctx, |e| e.avatar(Some(new_avatar.as_str())))
            .await?;
        channel
            .send_message(&ctx, |m| {
                m.reference_message(msg)
                    .allowed_mentions(|a| a.empty_parse())
                    .content(format!("Set avatar to {}", new_avatar))
            })
            .await?;
    }

    Ok(())
}
