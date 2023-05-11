use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use sparkle_convenience::Bot;
use tracing::{debug, error, info};
use twilight_gateway::stream::ShardRef;
use twilight_interactions::command::{CommandInputData, CommandModel};
use twilight_model::{
    application::{
        command::CommandType,
        interaction::{Interaction, InteractionData, InteractionType},
    },
    gateway::{
        payload::{
            incoming::{InteractionCreate, Ready},
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
        basic::{HelpCommand, InviteCommand, IssueCommand, SupportCommand},
        fuzzysearch::FuzzySearch,
        iqdb::Iqdb,
        saucenao::Saucenao,
    },
    Res,
};

pub async fn ready(mut shard: ShardRef<'_>, bot: Arc<Bot>, ready: Box<Ready>) -> Res<()> {
    let activity = Activity::from(MinimalActivity {
        kind: ActivityType::Playing,
        name: "/help - slash commands!".to_string(),
        url: None,
    });

    let command = UpdatePresence::new(vec![activity], false, None, Status::Online)?;

    shard.command(&command).await?;

    info!(
        "Shard {} ready, logged in as {}",
        shard.id(),
        ready.user.name
    );

    let commands = crate::commands::get();
    let interaction_client = bot.interaction_client();

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

pub async fn interaction_create(bot: Arc<Bot>, interaction: Box<InteractionCreate>) -> Res<()> {
    let interaction_id = interaction.id;
    let interaction = interaction.0;

    match interaction.kind {
        InteractionType::ApplicationCommand => {}
        _ => return Ok(()),
    };

    let data = interaction.data.clone();

    let Some(data) = data else {
        return Ok(());
    };

    let InteractionData::ApplicationCommand(data) = data else {
        return Ok(());
    };

    if matches!(data.kind, CommandType::Message | CommandType::User) {
        debug!("Unhandled kind: {:?}", data.kind);
        return Ok(());
    }

    let token = interaction.token.clone();
    let command_id = data.id;
    let name = data.name.clone();

    let input_data: CommandInputData = (*data).into();

    let cmd = Command {
        name: name.clone(),
        interaction_id,
        interaction,
        command_id,
        token,
    };

    before(&cmd);

    let res = {
        let ctx = bot.clone();
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

            "invite" => {
                let invite_command = InviteCommand::from_interaction(input_data)?;

                invite_command.execute(ctx, cmd).await
            }

            "iqdb" => {
                let iqdb_command = Iqdb::from_interaction(input_data)?;

                iqdb_command.execute(ctx, cmd).await
            }

            "saucenao" => {
                let saucenao_command = Saucenao::from_interaction(input_data)?;

                saucenao_command.execute(ctx, cmd).await
            }

            "fuzzysearch" => {
                let fuzzysearch_command = FuzzySearch::from_interaction(input_data)?;

                fuzzysearch_command.execute(ctx, cmd).await
            }

            _ => {
                debug!("Unhandled interaction: {}", name);

                return Ok(());
            }
        }
    };

    after(&cmd, res);

    Ok(())
}

fn before(cmd: &Command) {
    info!("Executing command {}", cmd.name);
}

fn after(cmd: &Command, res: Res<()>) {
    if let Err(e) = res {
        error!(?e, "Failed to execute {}", cmd.name);
    } else {
        info!("Successfully executed {}", cmd.name);
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub interaction_id: Id<InteractionMarker>,
    pub interaction: Interaction,
    pub command_id: Id<CommandMarker>,
    pub token: String,
}

impl Deref for Command {
    type Target = Interaction;

    fn deref(&self) -> &Self::Target {
        &self.interaction
    }
}

#[async_trait]
pub trait Cmd {
    async fn execute(&self, ctx: Arc<Bot>, command: Command) -> Res<()>;
}
