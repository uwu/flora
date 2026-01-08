use std::{cell::RefCell, rc::Rc, sync::Arc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    all::{CommandOptionType, CreateAttachment},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    http::Http,
    model::id::InteractionId,
};
use tracing::info;

use super::message::{
    RawAllowedMentions, RawAttachment, RawEmbed, build_allowed_mentions, build_attachment,
    build_embed,
};

#[expose_input]
pub(crate) struct RawInteractionResponse {
    pub interaction_id: String,
    pub token: String,
    pub content: Option<String>,
    pub embeds: Option<Vec<RawEmbed>>,
    pub attachments: Option<Vec<RawAttachment>>,
    pub tts: Option<bool>,
    pub allowed_mentions: Option<RawAllowedMentions>,
    pub ephemeral: Option<bool>,
}

#[expose_input]
pub(crate) struct RawUpsertGuildCommands {
    pub guild_id: String,
    pub commands: Vec<RawSlashCommand>,
}

#[expose_input]
pub(crate) struct RawSlashCommand {
    pub name: String,
    pub description: Option<String>,
    pub options: Option<Vec<RawSlashCommandOption>>,
}

#[expose_input]
pub(crate) struct RawSlashCommandOption {
    pub name: String,
    pub description: String,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default)]
    pub options: Option<Vec<RawSlashCommandOption>>,
}

#[op2(async)]
pub async fn op_send_interaction_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawInteractionResponse,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };

    let interaction_id = args
        .interaction_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid interaction id"))?;

    let built = build_interaction_response(&http, args).await?;

    let response = CreateInteractionResponse::Message(built.message);
    http.create_interaction_response(
        InteractionId::new(interaction_id),
        &built.token,
        &response,
        built.files,
    )
    .await
    .map_err(|err| JsErrorBox::generic(err.to_string()))?;

    Ok(())
}

#[op2(async)]
pub async fn op_upsert_guild_commands(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawUpsertGuildCommands,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };

    let guild_id =
        args.guild_id.parse::<u64>().map_err(|_| JsErrorBox::generic("Invalid guild id"))?;

    let command_defs = args.commands;
    let names: Vec<String> = command_defs.iter().map(|c| c.name.clone()).collect();
    let commands: Vec<CreateCommand> = command_defs
        .into_iter()
        .map(|cmd| {
            let desc = cmd
                .description
                .ok_or_else(|| JsErrorBox::generic("Slash command must have a description"))?;
            let mut builder = CreateCommand::new(cmd.name).description(desc);
            if let Some(options) = cmd.options {
                for opt in options {
                    builder = builder.add_option(build_option(opt)?);
                }
            }
            Ok(builder)
        })
        .collect::<Result<_, JsErrorBox>>()?;

    match http.create_guild_commands(serenity::model::id::GuildId::new(guild_id), &commands).await {
        Ok(_) => {
            info!(
                target: "flora:ops",
                guild_id,
                count = names.len(),
                commands = ?names,
                "successfully updated the slash commands on the guild"
            );
            Ok(())
        }
        Err(err) => {
            info!(
                target: "flora:ops",
                guild_id,
                ?err,
                commands = ?names,
                "failed to update slash commands"
            );
            Err(JsErrorBox::generic(err.to_string()))
        }
    }
}

fn build_option(opt: RawSlashCommandOption) -> Result<CreateCommandOption, JsErrorBox> {
    let opt_type = match opt.kind.as_deref() {
        Some("integer") => CommandOptionType::Integer,
        Some("number") => CommandOptionType::Number,
        Some("boolean") => CommandOptionType::Boolean,
        Some("subcommand") => CommandOptionType::SubCommand,
        Some("subcommand_group") => CommandOptionType::SubCommandGroup,
        _ => CommandOptionType::String,
    };

    let mut builder = CreateCommandOption::new(opt_type, opt.name, opt.description);
    if let Some(required) = opt.required {
        builder = builder.required(required);
    }
    if let Some(options) = opt.options {
        for nested_opt in options {
            builder = builder.add_sub_option(build_option(nested_opt)?);
        }
    }
    Ok(builder)
}

/// Build the response payload and attachments for an interaction reply.
pub(crate) struct BuiltInteractionResponse {
    pub message: CreateInteractionResponseMessage,
    pub token: String,
    pub files: Vec<CreateAttachment>,
}

pub(crate) async fn build_interaction_response(
    http: &Arc<Http>,
    args: RawInteractionResponse,
) -> Result<BuiltInteractionResponse, JsErrorBox> {
    let mut message = CreateInteractionResponseMessage::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut upload_files = Vec::new();

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds.into_iter().map(build_embed).collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(ephemeral) = args.ephemeral {
        if ephemeral {
            message = message.ephemeral(true);
        }
    }

    if let Some(attachments) = args.attachments {
        let mut files = Vec::with_capacity(attachments.len());
        for attachment in attachments {
            files.push(build_attachment(http, attachment).await?);
        }
        has_attachments = !files.is_empty();
        upload_files = files.clone();
        message = message.add_files(files);
    }

    if !has_content && !has_embeds && !has_attachments {
        return Err(JsErrorBox::generic("Response must include content, embeds, or attachments"));
    }

    Ok(BuiltInteractionResponse { message, token: args.token, files: upload_files })
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::*;

    fn http() -> Arc<Http> {
        Arc::new(Http::new("test"))
    }

    #[tokio::test]
    async fn build_rejects_empty_payload() {
        let args = RawInteractionResponse {
            interaction_id: "1".to_string(),
            token: "token".to_string(),
            content: None,
            embeds: None,
            attachments: None,
            tts: None,
            allowed_mentions: None,
            ephemeral: None,
        };

        let result = build_interaction_response(&http(), args).await;
        assert!(result.is_err(), "expected empty payload to be rejected");
    }

    #[tokio::test]
    async fn build_allows_base64_attachment_and_ephemeral() {
        let data = STANDARD.encode(b"hello");
        let args = RawInteractionResponse {
            interaction_id: "1".to_string(),
            token: "token".to_string(),
            content: Some("hi".to_string()),
            embeds: None,
            attachments: Some(vec![RawAttachment::Base64 {
                data,
                filename: "greet.txt".to_string(),
                description: Some("greeting".to_string()),
            }]),
            tts: Some(false),
            allowed_mentions: None,
            ephemeral: Some(true),
        };

        let built = build_interaction_response(&http(), args).await.unwrap();
        assert_eq!(built.files.len(), 1);

        let value = serde_json::to_value(&built.message).unwrap();
        let flags = value.get("flags").and_then(|v| v.as_u64()).unwrap_or(0);
        // 1 << 6 is the ephemeral flag per Discord docs.
        assert_eq!(flags & (1 << 6), 1 << 6, "expected ephemeral flag to be set");
    }
}
