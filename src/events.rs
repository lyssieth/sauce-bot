use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::time::sleep;
use tracing::{debug, error, info};
use twilight_interactions::command::{CommandInputData, CommandModel};
use twilight_model::{
    application::{
        command::CommandType,
        interaction::{InteractionData, InteractionType},
    },
    gateway::{
        payload::{
            incoming::{InteractionCreate, MessageCreate, Ready},
            outgoing::UpdatePresence,
        },
        presence::{Activity, ActivityType, MinimalActivity, Status},
    },
    guild::Permissions,
    id::{
        marker::{CommandMarker, InteractionMarker},
        Id,
    },
};

use crate::{
    commands::{
        basic::{HelpCommand, IssueCommand, SupportCommand},
        iqdb::Iqdb,
        saucenao::Saucenao,
    },
    Context, Res,
};

pub async fn ready(shard_id: u64, ctx: Arc<Context>, ready: Box<Ready>) -> Res<()> {
    let activity = Activity::from(MinimalActivity {
        kind: ActivityType::Playing,
        name: "/help - slash commands!".to_string(),
        url: None,
    });

    let command = UpdatePresence::new(vec![activity], false, None, Status::Online)?;

    ctx.cluster.command(shard_id, &command).await?;

    info!("Shard {} ready, logged in as {}", shard_id, ready.user.name);

    let commands = crate::commands::get();
    let interaction_client = ctx.interaction_client();

    for x in commands {
        let cg = interaction_client
            .create_global_command()
            .chat_input(&x.name, &x.description)?
            .command_options(&x.options)?
            .default_member_permissions(
                x.default_member_permissions.unwrap_or(Permissions::empty()),
            )
            .dm_permission(true);

        match cg.await {
            Ok(cmd) => {
                let cmd = cmd.model().await?;
                debug!("Created command {}", cmd.name);
            }
            Err(e) => {
                error!("Failed to create command {}: {e}", x.name);
            }
        }
    }

    Ok(())
}

pub async fn message_create(
    _shard_id: u64,
    ctx: Arc<Context>,
    message: Box<MessageCreate>,
) -> Res<()> {
    if !message.content.starts_with("sauce!") {
        return Ok(());
    }

    let msg = ctx
        .http
        .create_message(message.channel_id)
        .content("SauceBot now uses slash commands.\n\nThis message will delete after 20 seconds.")?
        .reply(message.id)
        .await?
        .model()
        .await?;
    let msg = Arc::new(msg);

    tokio::spawn(async move {
        sleep(Duration::from_secs(20)).await;
        drop(ctx.http.delete_message(msg.channel_id, msg.id).await);
    });

    Ok(())
}

pub async fn interaction_create(
    shard_id: u64,
    ctx: Arc<Context>,
    interaction: Box<InteractionCreate>,
) -> Res<()> {
    let interaction_id = interaction.id;
    let interaction = interaction.0;

    match interaction.kind {
        InteractionType::ApplicationCommand => {}
        _ => return Ok(()),
    };

    let data = interaction.data;

    let data = match data {
        Some(data) => data,
        None => return Ok(()),
    };

    let data = match data {
        InteractionData::ApplicationCommand(data) => data,
        _ => return Ok(()),
    };

    if matches!(data.kind, CommandType::Message | CommandType::User) {
        debug!("Unhandled kind: {:?}", data.kind);
        return Ok(());
    }

    let token = interaction.token;
    let command_id = data.id;
    let name = data.name.clone();

    let input_data: CommandInputData = (*data).into();

    let cmd = Command {
        name: name.clone(),
        interaction_id,
        command_id,
        token,
    };

    before(shard_id, &cmd);

    let res = {
        let ctx = ctx.clone();
        let cmd = cmd.clone();

        match name.as_str() {
            "help" => {
                let help_command = HelpCommand::from_interaction(input_data)?;

                help_command.execute(ctx, cmd).await
            }

            "issue" => {
                let issue_command = IssueCommand::from_interaction(input_data)?;

                issue_command.execute(ctx, cmd).await
            }

            "support" => {
                let support_command = SupportCommand::from_interaction(input_data)?;

                support_command.execute(ctx, cmd).await
            }

            "iqdb" => {
                let iqdb_command = Iqdb::from_interaction(input_data)?;

                iqdb_command.execute(ctx, cmd).await
            }

            "saucenao" => {
                let saucenao_command = Saucenao::from_interaction(input_data)?;

                saucenao_command.execute(ctx, cmd).await
            }

            _ => {
                debug!("Unhandled interaction: {}", name);

                return Ok(());
            }
        }
    };

    after(shard_id, &cmd, res);

    Ok(())
}

fn before(shard_id: u64, cmd: &Command) {
    info!("Executing command {} on shard {}", cmd.name, shard_id);
}

fn after(shard_id: u64, cmd: &Command, res: Res<()>) {
    if let Err(e) = res {
        error!(?e, "Failed to execute {} on shard {}", cmd.name, shard_id);
    } else {
        info!("Successfully executed {} on shard {}", cmd.name, shard_id);
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub interaction_id: Id<InteractionMarker>,
    pub command_id: Id<CommandMarker>,
    pub token: String,
}

#[async_trait]
pub trait Cmd {
    async fn execute(&self, ctx: Arc<Context>, command: Command) -> Res<()>;
}
