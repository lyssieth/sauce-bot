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

use crate::{commands::basic::HelpCommand, Context, Res};

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

#[allow(clippy::unused_async, unused)]
pub(crate) async fn interaction_create(
    shard_id: u64,
    ctx: Arc<Context>,
    interaction: Box<InteractionCreate>,
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

    match name.as_str() {
        "help" => {
            let help_command = HelpCommand::from_interaction(input_data)?;

            let cmd = Command {
                name,
                interaction_id,
                command_id,
                token,
            };

            help_command.execute(ctx, cmd).await?;
        }

        _ => {
            debug!("Unhandled interaction: {}", name);
        }
    }

    Ok(())
}

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
