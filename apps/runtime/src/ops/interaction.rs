use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    all::{CommandOptionType, CreateAttachment},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
        EditInteractionResponse,
    },
    http::Http,
    model::{channel::MessageFlags, id::InteractionId},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
    sync::Arc,
};
use t0x::T0x;
use tracing::info;

use super::components::parse_components;
use super::message::{
    RawAllowedMentions, RawAttachment, RawEmbed, build_allowed_mentions, build_attachment,
    build_embed,
};

/// Arguments for sending an initial interaction response.
#[expose_input]
pub struct RawInteractionResponse {
    /// The interaction's snowflake ID.
    pub interaction_id: String,
    /// Token for this interaction.
    pub token: String,
    /// Message content.
    pub content: Option<String>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Attachments to include.
    pub attachments: Option<Vec<RawAttachment>>,
    /// Message components (buttons, select menus).
    pub components: Option<Vec<serde_json::Value>>,
    /// Whether the message should be text-to-speech.
    pub tts: Option<bool>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Whether the response should be ephemeral (only visible to invoker).
    pub ephemeral: Option<bool>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
}

/// Arguments for bulk-upserting guild application commands.
#[derive(serde::Serialize)]
#[expose_input]
pub struct RawUpsertGuildCommands {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The commands to register.
    pub commands: Vec<RawSlashCommand>,
}

/// Definition of a slash command.
#[derive(serde::Serialize)]
#[expose_input]
pub struct RawSlashCommand {
    /// The command name (1-32 chars, lowercase).
    pub name: String,
    /// The command description (1-100 chars).
    pub description: Option<String>,
    /// Options/arguments for the command.
    pub options: Option<Vec<RawSlashCommandOption>>,
}

/// Definition of a slash command option.
#[derive(serde::Serialize)]
#[expose_input]
pub struct RawSlashCommandOption {
    /// The option name.
    pub name: String,
    /// The option description.
    pub description: String,
    /// The option type (string, integer, boolean, etc.).
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Whether this option is required.
    #[serde(default)]
    pub required: Option<bool>,
    /// Nested options (for subcommands/subcommand groups).
    #[serde(default)]
    #[t0x(type = "RawSlashCommandOption[]")]
    pub options: Option<Vec<RawSlashCommandOption>>,
}

/// Memoizes the last slash-command payload per guild to avoid redundant upserts.
#[derive(Default)]
pub struct CommandHashCache {
    by_guild: HashMap<String, u64>,
}

impl CommandHashCache {
    pub fn is_duplicate_and_update(&mut self, guild_id: &str, hash: u64) -> bool {
        if self.by_guild.get(guild_id) == Some(&hash) {
            true
        } else {
            self.by_guild.insert(guild_id.to_string(), hash);
            false
        }
    }
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

/// Arguments for deferring an interaction response.
#[expose_input]
pub struct RawDeferInteractionResponse {
    /// The interaction's snowflake ID.
    pub interaction_id: String,
    /// Token for this interaction.
    pub token: String,
    /// Whether the deferred response should be ephemeral.
    pub ephemeral: Option<bool>,
}

#[op2(async)]
pub async fn op_defer_interaction_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeferInteractionResponse,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let interaction_id = args
        .interaction_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid interaction id"))?;
    let mut message = CreateInteractionResponseMessage::new();
    if let Some(ephemeral) = args.ephemeral {
        message = message.ephemeral(ephemeral);
    }
    let response = CreateInteractionResponse::Defer(message);
    http.create_interaction_response(
        InteractionId::new(interaction_id),
        &args.token,
        &response,
        Vec::new(),
    )
    .await
    .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

/// Arguments for updating a component interaction's message.
#[expose_input]
pub struct RawUpdateInteractionResponse {
    /// The interaction's snowflake ID.
    pub interaction_id: String,
    /// Token for this interaction.
    pub token: String,
    /// New message content.
    pub content: Option<String>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Attachments to include.
    pub attachments: Option<Vec<RawAttachment>>,
    /// Message components.
    pub components: Option<Vec<serde_json::Value>>,
    /// Whether the message should be text-to-speech.
    pub tts: Option<bool>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
}

#[op2(async)]
pub async fn op_update_interaction_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawUpdateInteractionResponse,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let interaction_id = args
        .interaction_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid interaction id"))?;

    let built = build_interaction_update(&http, args).await?;
    let response = CreateInteractionResponse::UpdateMessage(built.message);
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

/// Arguments for editing the original interaction response.
#[expose_input]
pub struct RawEditInteractionResponse {
    /// Token for this interaction.
    pub token: String,
    /// New message content.
    pub content: Option<String>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Attachments to include.
    pub attachments: Option<Vec<RawAttachment>>,
    /// Message components.
    pub components: Option<Vec<serde_json::Value>>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_original_interaction_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditInteractionResponse,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let built = build_edit_interaction_response(&http, args).await?;
    let message = http
        .edit_original_interaction_response(&built.token, &built.message, built.files)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for deleting the original interaction response.
#[expose_input]
pub struct RawDeleteInteractionResponse {
    /// Token for this interaction.
    pub token: String,
}

#[op2(async)]
pub async fn op_delete_original_interaction_response(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteInteractionResponse,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    http.delete_original_interaction_response(&args.token)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

/// Arguments for creating or editing a followup message.
#[expose_input]
pub struct RawFollowupMessage {
    /// Token for this interaction.
    pub token: String,
    /// Message ID (required when editing a followup).
    pub message_id: Option<String>,
    /// Message content.
    pub content: Option<String>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Attachments to include.
    pub attachments: Option<Vec<RawAttachment>>,
    /// Message components.
    pub components: Option<Vec<serde_json::Value>>,
    /// Whether the message should be text-to-speech.
    pub tts: Option<bool>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
}

#[op2(async)]
#[serde]
pub async fn op_create_followup_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawFollowupMessage,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let built = build_followup_message(&http, args).await?;
    let message = http
        .create_followup_message(&built.token, &built.message, built.files)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[op2(async)]
#[serde]
pub async fn op_edit_followup_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawFollowupMessage,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let message_id = args
        .message_id
        .as_ref()
        .ok_or_else(|| JsErrorBox::generic("message_id required to edit followup"))?;
    let message_id = message_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid message id"))?;
    let built = build_followup_message(&http, args).await?;
    let message = http
        .edit_followup_message(
            &built.token,
            serenity::model::id::MessageId::new(message_id),
            &built.message,
            built.files,
        )
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub struct RawDeleteFollowupMessage {
    pub token: String,
    pub message_id: String,
}

#[op2(async)]
pub async fn op_delete_followup_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteFollowupMessage,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let message_id = args
        .message_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid message id"))?;
    http.delete_followup_message(&args.token, serenity::model::id::MessageId::new(message_id))
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_upsert_guild_commands(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawUpsertGuildCommands,
) -> Result<(), JsErrorBox> {
    let command_defs = args.commands;
    let commands_hash = serde_json::to_string(&command_defs)
        .map(hash_json)
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;

    let (http, skipped) = {
        let mut state = state.borrow_mut();
        let http = state.borrow::<Arc<Http>>().clone();
        let cache = state.borrow_mut::<CommandHashCache>();
        let skipped = cache.is_duplicate_and_update(&args.guild_id, commands_hash);
        (http, skipped)
    };

    if skipped {
        info!(
            target: "flora:ops",
            guild_id = args.guild_id,
            "slash commands unchanged; skipping upsert"
        );
        return Ok(());
    }

    let guild_id = args
        .guild_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid guild id"))?;

    let names: Vec<String> = command_defs.iter().map(|c| c.name.clone()).collect();
    let commands: Vec<CreateCommand<'static>> = command_defs
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

    match http
        .create_guild_commands(serenity::model::id::GuildId::new(guild_id), &commands)
        .await
    {
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

fn build_option(opt: RawSlashCommandOption) -> Result<CreateCommandOption<'static>, JsErrorBox> {
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

fn hash_json(value: String) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Build the response payload and attachments for an interaction reply.
pub(crate) struct BuiltInteractionResponse {
    pub message: CreateInteractionResponseMessage<'static>,
    pub token: String,
    pub files: Vec<CreateAttachment<'static>>,
}

pub(crate) async fn build_interaction_response(
    http: &Arc<Http>,
    args: RawInteractionResponse,
) -> Result<BuiltInteractionResponse, JsErrorBox> {
    let mut message = CreateInteractionResponseMessage::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut has_components = false;
    let mut upload_files = Vec::new();

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds
            .into_iter()
            .map(build_embed)
            .collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(components) = args.components {
        let components = parse_components(components)?;
        message = message.components(components);
        has_components = true;
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(ephemeral) = args.ephemeral {
        if ephemeral {
            message = message.ephemeral(true);
        }
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
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

    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(JsErrorBox::generic(
            "Response must include content, embeds, attachments, or components",
        ));
    }

    Ok(BuiltInteractionResponse {
        message,
        token: args.token,
        files: upload_files,
    })
}

pub(crate) struct BuiltInteractionUpdate {
    pub message: CreateInteractionResponseMessage<'static>,
    pub token: String,
    pub files: Vec<CreateAttachment<'static>>,
}

pub(crate) async fn build_interaction_update(
    http: &Arc<Http>,
    args: RawUpdateInteractionResponse,
) -> Result<BuiltInteractionUpdate, JsErrorBox> {
    let mut message = CreateInteractionResponseMessage::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut has_components = false;
    let mut upload_files = Vec::new();

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds
            .into_iter()
            .map(build_embed)
            .collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(components) = args.components {
        let components = parse_components(components)?;
        message = message.components(components);
        has_components = true;
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
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

    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(JsErrorBox::generic(
            "Response must include content, embeds, attachments, or components",
        ));
    }

    Ok(BuiltInteractionUpdate {
        message,
        token: args.token,
        files: upload_files,
    })
}

pub(crate) struct BuiltEditInteractionResponse {
    pub message: EditInteractionResponse<'static>,
    pub token: String,
    pub files: Vec<CreateAttachment<'static>>,
}

pub(crate) async fn build_edit_interaction_response(
    http: &Arc<Http>,
    args: RawEditInteractionResponse,
) -> Result<BuiltEditInteractionResponse, JsErrorBox> {
    let mut message = EditInteractionResponse::new();
    let mut has_payload = false;
    let mut upload_files = Vec::new();

    if let Some(content) = args.content {
        message = message.content(content);
        has_payload = true;
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds
            .into_iter()
            .map(build_embed)
            .collect::<Result<Vec<_>, _>>()?;
        message = message.embeds(embeds);
        has_payload = true;
    }

    if let Some(components) = args.components {
        let components = parse_components(components)?;
        message = message.components(components);
        has_payload = true;
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
        has_payload = true;
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
        has_payload = true;
    }

    if let Some(attachments) = args.attachments {
        let mut files = Vec::with_capacity(attachments.len());
        for attachment in attachments {
            files.push(build_attachment(http, attachment).await?);
        }
        upload_files = files.clone();
        let mut edit = serenity::builder::EditAttachments::new();
        for file in files {
            edit = edit.add(file);
        }
        message = message.attachments(edit);
        has_payload = true;
    }

    if !has_payload {
        return Err(JsErrorBox::generic(
            "Edit response must include content, embeds, components, attachments, or flags",
        ));
    }

    Ok(BuiltEditInteractionResponse {
        message,
        token: args.token,
        files: upload_files,
    })
}

pub(crate) struct BuiltFollowupMessage {
    pub message: CreateInteractionResponseFollowup<'static>,
    pub token: String,
    pub files: Vec<CreateAttachment<'static>>,
}

pub(crate) async fn build_followup_message(
    http: &Arc<Http>,
    args: RawFollowupMessage,
) -> Result<BuiltFollowupMessage, JsErrorBox> {
    let mut message = CreateInteractionResponseFollowup::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut has_components = false;
    let mut upload_files = Vec::new();

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds
            .into_iter()
            .map(build_embed)
            .collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(components) = args.components {
        let components = parse_components(components)?;
        message = message.components(components);
        has_components = true;
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
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

    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(JsErrorBox::generic(
            "Followup must include content, embeds, attachments, or components",
        ));
    }

    Ok(BuiltFollowupMessage {
        message,
        token: args.token,
        files: upload_files,
    })
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::*;

    fn http() -> Arc<Http> {
        Arc::new(Http::without_token())
    }

    #[tokio::test]
    async fn build_rejects_empty_payload() {
        let args = RawInteractionResponse {
            interaction_id: "1".to_string(),
            token: "token".to_string(),
            content: None,
            embeds: None,
            attachments: None,
            components: None,
            tts: None,
            allowed_mentions: None,
            ephemeral: None,
            flags: None,
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
            components: None,
            tts: Some(false),
            allowed_mentions: None,
            ephemeral: Some(true),
            flags: None,
        };

        let built = build_interaction_response(&http(), args).await.unwrap();
        assert_eq!(built.files.len(), 1);

        let value = serde_json::to_value(&built.message).unwrap();
        let flags = value.get("flags").and_then(|v| v.as_u64()).unwrap_or(0);
        // 1 << 6 is the ephemeral flag per Discord docs.
        assert_eq!(
            flags & (1 << 6),
            1 << 6,
            "expected ephemeral flag to be set"
        );
    }
}
