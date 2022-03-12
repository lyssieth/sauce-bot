use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, error, info};
use twilight_interactions::command::{CommandInputData, CommandModel};
use twilight_model::{
    application::{command::CommandType, interaction::Interaction},
    gateway::{
        payload::{
            incoming::{InteractionCreate, Ready},
            outgoing::UpdatePresence,
        },
        presence::{Activity, ActivityType, MinimalActivity, Status},
    },
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

pub(crate) async fn ready(shard_id: u64, ctx: Arc<Context>, ready: Box<Ready>) -> Res<()> {
    let activity = Activity::from(MinimalActivity {
        kind: ActivityType::Custom,
        name: "sauce!help".to_string(),
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
            .default_permission(x.default_permission)
            .exec();

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

pub(crate) async fn interaction_create(
    shard_id: u64,
    ctx: Arc<Context>,
    interaction: InteractionCreate,
) -> Res<()> {
    let interaction_id = interaction.id();
    let application_command = match interaction.0 {
        Interaction::ApplicationCommand(cmd) => cmd,
        _ => return Ok(()),
    };
    let data = application_command.data;

    if matches!(data.kind, CommandType::Message | CommandType::User) {
        debug!("Unhandled kind: {:?}", data.kind);
        return Ok(());
    }

    let token = application_command.token;
    let command_id = data.id;
    let name = data.name.clone();

    let input_data: CommandInputData = data.into();

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
