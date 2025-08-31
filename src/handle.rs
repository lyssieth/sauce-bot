use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use sparkle_convenience::{
    Bot,
    error::{Error, UserError},
    reply::Reply,
};
use twilight_model::{
    application::{
        command::CommandOptionChoice,
        interaction::{Interaction, InteractionType},
    },
    channel::{
        Message,
        message::{
            Component,
            component::{ActionRow, TextInput},
        },
    },
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{
        Id,
        marker::{InteractionMarker, MessageMarker},
    },
};

/// A reimplementation of [`sparkle_convenience::interaction::InteractionHandle`], but with an Arc.
#[derive(Debug, Clone)]
pub struct Handle {
    bot: Arc<Bot>,
    id: Id<InteractionMarker>,
    token: String,
    kind: InteractionType,
    app_permissions: Permissions,
    responded: Arc<AtomicBool>,
    last_message_id: Arc<AtomicU64>,
}

pub trait SpecialHandler {
    fn handle(&self, interaction: &Interaction) -> Handle;
}

impl SpecialHandler for Arc<Bot> {
    fn handle(&self, interaction: &Interaction) -> Handle {
        Handle {
            bot: self.clone(),
            id: interaction.id,
            token: interaction.token.clone(),
            kind: interaction.kind,
            app_permissions: interaction.app_permissions.unwrap_or(Permissions::all()),
            responded: Arc::new(AtomicBool::new(false)),
            last_message_id: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[allow(unused)]
impl Handle {
    pub const fn check_permissions<C>(
        &self,
        required_permissions: Permissions,
    ) -> Result<(), UserError<C>> {
        let missing_permissions = required_permissions.difference(self.app_permissions);
        if !missing_permissions.is_empty() {
            return Err(UserError::MissingPermissions(Some(missing_permissions)));
        }

        Ok(())
    }

    pub async fn report_error<C: Send>(
        &self,
        reply: Reply,
        error: UserError<C>,
    ) -> Result<Option<Message>, Error> {
        if matches!(error, UserError::Ignore) {
            return Ok(None);
        }

        match self.reply(reply).await {
            Ok(message) => Ok(message),
            Err(Error::Http(err))
                if matches!(UserError::<C>::from_http_err(&err), UserError::Internal) =>
            {
                Err(Error::Http(err))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn reply(&self, reply: Reply) -> Result<Option<Message>, Error> {
        let client = self.bot.reply_handle(&reply);
        let interaction = self.bot.http.interaction(self.bot.application.id);
        if self.responded() {
            if reply.update_last {
                if let Some(last_message_id) = self.last_message_id() {
                    let mut update_followup =
                        interaction.update_followup(&self.token, last_message_id);

                    if let Some(allowed_mentions) = &reply.allowed_mentions {
                        update_followup =
                            update_followup.allowed_mentions(allowed_mentions.as_ref());
                    }
                    update_followup
                        .content((!reply.content.is_empty()).then_some(&reply.content))
                        .embeds(Some(&reply.embeds))
                        .components(Some(&reply.components))
                        .attachments(&reply.attachments)
                        .await?;

                    Ok(None)
                } else {
                    let mut update_response = interaction.update_response(&self.token);

                    if let Some(allowed_mentions) = &reply.allowed_mentions {
                        update_response =
                            update_response.allowed_mentions(allowed_mentions.as_ref());
                    }

                    let message = update_response
                        .content((!reply.content.is_empty()).then_some(&reply.content))
                        .embeds(Some(&reply.embeds))
                        .components(Some(&reply.components))
                        .attachments(&reply.attachments)
                        .await?
                        .model()
                        .await?;

                    self.set_last_message_id(message.id);

                    Ok(Some(message))
                }
            } else {
                let mut followup = interaction.create_followup(&self.token);

                if !reply.content.is_empty() {
                    followup = followup.content(&reply.content);
                }
                if let Some(allowed_mentions) = &reply.allowed_mentions {
                    followup = followup.allowed_mentions(allowed_mentions.as_ref());
                }

                let message = followup
                    .embeds(&reply.embeds)
                    .components(&reply.components)
                    .attachments(&reply.attachments)
                    .flags(reply.flags)
                    .tts(reply.tts)
                    .await?
                    .model()
                    .await?;

                self.set_last_message_id(message.id);

                Ok(Some(message))
            }
        } else {
            let kind = if reply.update_last && self.kind == InteractionType::MessageComponent {
                InteractionResponseType::UpdateMessage
            } else {
                InteractionResponseType::ChannelMessageWithSource
            };

            interaction
                .create_response(
                    self.id,
                    &self.token,
                    &InteractionResponse {
                        kind,
                        data: Some(reply.into()),
                    },
                )
                .await?;

            self.set_responded(true);

            Ok(None)
        }
    }

    pub async fn autocomplete(&self, choices: Vec<CommandOptionChoice>) -> Result<(), Error> {
        if self.responded() {
            return Err(Error::AlreadyResponded);
        }

        let interaction = self.bot.http.interaction(self.bot.application.id);

        interaction
            .create_response(
                self.id,
                &self.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
                    data: Some(InteractionResponseData {
                        choices: Some(choices),
                        ..Default::default()
                    }),
                },
            )
            .await?;

        self.set_responded(true);

        Ok(())
    }

    pub async fn modal(
        &self,
        custom_id: impl Into<String> + Send,
        title: impl Into<String> + Send,
        text_inputs: Vec<TextInput>,
    ) -> Result<(), Error> {
        let responded = self.responded();

        if responded {
            return Err(Error::AlreadyResponded);
        }

        let interaction = self.bot.http.interaction(self.bot.application.id);

        interaction
            .create_response(
                self.id,
                &self.token,
                &InteractionResponse {
                    kind: InteractionResponseType::Modal,
                    data: Some(InteractionResponseData {
                        custom_id: Some(custom_id.into()),
                        title: Some(title.into()),
                        components: Some(
                            text_inputs
                                .into_iter()
                                .map(|text_input| {
                                    Component::ActionRow(ActionRow {
                                        components: vec![Component::TextInput(text_input)],
                                    })
                                })
                                .collect(),
                        ),
                        ..Default::default()
                    }),
                },
            )
            .await?;

        self.set_responded(true);

        Ok(())
    }

    fn responded(&self) -> bool {
        self.responded.load(Ordering::Acquire)
    }

    fn set_responded(&self, val: bool) {
        self.responded.store(val, Ordering::Release);
    }

    fn last_message_id(&self) -> Option<Id<MessageMarker>> {
        let id = self.last_message_id.load(Ordering::Acquire);
        if id == 0 { None } else { Some(Id::new(id)) }
    }

    fn set_last_message_id(&self, val: Id<MessageMarker>) {
        self.last_message_id.store(val.get(), Ordering::Release);
    }
}
