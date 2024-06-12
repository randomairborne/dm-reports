use std::ops::Deref;

use axum::{
    body::Bytes,
    extract::{FromRequest, State},
    Json,
};
use serde_json::json;
use twilight_http::{request::Request, routing::Route};
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::{Interaction, InteractionData, InteractionType},
    },
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::Id,
};
use twilight_util::builder::{
    command::CommandBuilder,
    embed::{EmbedBuilder, EmbedFieldBuilder},
    InteractionResponseDataBuilder,
};

use crate::{
    validate_signature::{SIGNATURE_HEADER, TIMESTAMP_HEADER},
    AppState, Error as HttpError,
};

const REPORT_COMMAND_NAME: &str = "Report to Minecraft Discord";

pub fn interaction_message(description: String) -> InteractionResponse {
    let embed = EmbedBuilder::new().description(description).build();

    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .embeds([embed])
        .build();

    InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(data),
    }
}

pub struct ExtractInteraction(pub Interaction);

impl Deref for ExtractInteraction {
    type Target = Interaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[axum::async_trait]
impl FromRequest<AppState> for ExtractInteraction {
    type Rejection = HttpError;

    async fn from_request(
        req: axum::extract::Request,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let signature = req
            .headers()
            .get(SIGNATURE_HEADER)
            .ok_or(HttpError::MissingHeader(SIGNATURE_HEADER))?
            .clone();
        let timestamp = req
            .headers()
            .get(TIMESTAMP_HEADER)
            .ok_or(HttpError::MissingHeader(TIMESTAMP_HEADER))?
            .clone();

        let body = Bytes::from_request(req, &()).await?;

        state
            .verify_key
            .verify(signature.as_bytes(), timestamp.as_bytes(), body.as_ref())?;
        Ok(Self(serde_json::from_slice(&body)?))
    }
}

#[instrument(skip(state))]
pub async fn interact(
    State(state): State<AppState>,
    ExtractInteraction(interaction): ExtractInteraction,
) -> Json<InteractionResponse> {
    let resp = match interaction.kind {
        InteractionType::Ping => InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None,
        },
        InteractionType::ApplicationCommand => interaction_message(
            command(state, interaction)
                .await
                .unwrap_or_else(|e| e.to_string()),
        ),
        _ => unsupported(),
    };
    Json(resp)
}

fn unsupported() -> InteractionResponse {
    interaction_message("Unsupported interaction kind".to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Contacting discord returned an error")]
    Discord(#[from] twilight_http::Error),
    #[error("Failed to serialize commands")]
    Json(#[from] serde_json::Error),
    #[error("Bad interaction data")]
    BadInteractionData,
    #[error("No resolved data")]
    NoResolvedData,
    #[error("No target ID")]
    NoTargetId,
    #[error("Missing message in resolved data")]
    MissingMessage,
    #[error("No interaction invoker!")]
    NoInteractionInvoker,
    #[error("You can't report your own message!")]
    CantReportOwnMessage,
}

fn created_at<T>(id: Id<T>) -> u64 {
    ((id.get() >> 22) + 1420070400000) / 1000
}

async fn command(state: AppState, interaction: Interaction) -> Result<String, Error> {
    let invoker = interaction
        .author()
        .ok_or(Error::NoInteractionInvoker)?
        .clone();
    let Some(InteractionData::ApplicationCommand(data)) = interaction.data else {
        return Err(Error::BadInteractionData);
    };
    let resolved = data.resolved.ok_or(Error::NoResolvedData)?;
    let target = data.target_id.ok_or(Error::NoTargetId)?;

    let message = resolved
        .messages
        .get(&target.cast())
        .ok_or(Error::MissingMessage)?;

    if message.author.id == invoker.id {
        return Err(Error::CantReportOwnMessage);
    }

    let author = EmbedFieldBuilder::new("author", format!("<@{}>", message.author.id))
        .inline()
        .build();
    let invoker = EmbedFieldBuilder::new("reporter", format!("<@{}>", invoker.id))
        .inline()
        .build();
    let edited = EmbedFieldBuilder::new(
        "edited",
        message
            .edited_timestamp
            .map_or_else(|| "never".to_string(), |t| format!("<t:{}:R>", t.as_secs())),
    )
    .inline()
    .build();
    let sent_at = EmbedFieldBuilder::new("sent", format!("<t:{}:R>", created_at(message.id)))
        .inline()
        .build();

    let embed = EmbedBuilder::new()
        .field(author)
        .field(invoker)
        .field(edited)
        .field(sent_at)
        .build();

    state
        .client
        .execute_webhook(state.config.hook.id, state.config.hook.token.as_str())
        .content(&message.content)
        .embeds(&[embed])
        .await?;
    Ok("Report submitted. Thank you!".to_string())
}

#[instrument(skip_all)]
pub async fn register_commands(state: &AppState) -> Result<(), Error> {
    // This horribleness brought to you by Advaith and Discord's fucking horrendous GA policies
    let command_struct = CommandBuilder::new(REPORT_COMMAND_NAME, "", CommandType::Message).build();
    let mut command_value = serde_json::to_value(command_struct)?;
    let Some(cmd_value_object) = command_value.as_object_mut() else {
        unreachable!("Serializing a struct and getting not-a-map should be impossible");
    };

    cmd_value_object.insert("integration_types".to_string(), json!([1]));
    cmd_value_object.insert("contexts".to_string(), json!([2]));

    let request = Request::builder(&Route::SetGlobalCommands {
        application_id: state.application_id.get(),
    })
    .json(&json!([cmd_value_object]))
    .build()?;
    state.client.request::<Vec<Command>>(request).await?;
    Ok(())
}
