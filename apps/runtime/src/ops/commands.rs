use super::{
    authz::ensure_guild_scope,
    interaction::{RawSlashCommand, RawSlashCommandOption},
};
use crate::services::discord_rest::{DiscordRest, RestRetry};
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    builder::{CreateCommand, CreateCommandOption},
    model::id::{CommandId, GuildId},
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use t0x::T0x;

use super::FloraError;

/// Arguments for creating a guild application command.
#[expose_input]
pub struct RawCreateGuildCommand {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The command to create.
    pub command: RawSlashCommand,
}

#[op2(async)]
#[serde]
pub async fn op_create_guild_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateGuildCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let command = build_command(args.command)?;
    let route = format!("POST /guilds/{}/commands", guild_id.get());
    let created = rest
        .execute(guild_id, route, RestRetry::None, move |http| {
            let command = command.clone();
            async move { http.create_guild_command(guild_id, &command).await }
        })
        .await?;
    serde_json::to_value(created).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing a guild application command.
#[expose_input]
pub struct RawEditGuildCommand {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The command's snowflake ID.
    pub command_id: String,
    /// Updated command data.
    pub command: RawSlashCommand,
}

#[op2(async)]
#[serde]
pub async fn op_edit_guild_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditGuildCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let command_id = parse_command_id(&args.command_id)?;
    let command = build_command(args.command)?;
    let route = format!(
        "PATCH /guilds/{}/commands/{}",
        guild_id.get(),
        command_id.get()
    );
    let updated = rest
        .execute(guild_id, route, RestRetry::None, move |http| {
            let command = command.clone();
            async move {
                http.edit_guild_command(guild_id, command_id, &command)
                    .await
            }
        })
        .await?;
    serde_json::to_value(updated).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for deleting a guild application command.
#[expose_input]
pub struct RawDeleteGuildCommand {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The command's snowflake ID.
    pub command_id: String,
}

#[op2(async)]
pub async fn op_delete_guild_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteGuildCommand,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let command_id = parse_command_id(&args.command_id)?;
    let route = format!(
        "DELETE /guilds/{}/commands/{}",
        guild_id.get(),
        command_id.get()
    );
    rest.execute(guild_id, route, RestRetry::None, move |http| async move {
        http.delete_guild_command(guild_id, command_id).await
    })
    .await?;
    Ok(())
}

/// Arguments for fetching a guild application command.
#[expose_input]
pub struct RawGetGuildCommand {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The command's snowflake ID.
    pub command_id: String,
}

#[op2(async)]
#[serde]
pub async fn op_get_guild_commands(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildId,
) -> Result<Vec<serde_json::Value>, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let route = format!("GET /guilds/{}/commands", guild_id.get());
    let commands = rest
        .execute(
            guild_id,
            route,
            RestRetry::ReadOnly,
            move |http| async move { http.get_guild_commands(guild_id).await },
        )
        .await?;
    commands
        .into_iter()
        .map(|cmd| serde_json::to_value(cmd).map_err(|err| JsErrorBox::generic(err.to_string())))
        .collect()
}

#[op2(async)]
#[serde]
pub async fn op_get_guild_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGetGuildCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let command_id = parse_command_id(&args.command_id)?;
    let route = format!(
        "GET /guilds/{}/commands/{}",
        guild_id.get(),
        command_id.get()
    );
    let command = rest
        .execute(
            guild_id,
            route,
            RestRetry::ReadOnly,
            move |http| async move { http.get_guild_command(guild_id, command_id).await },
        )
        .await?;
    serde_json::to_value(command).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing guild command permissions.
#[expose_input]
pub struct RawCommandPermissions {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The command's snowflake ID.
    pub command_id: String,
    /// New permissions payload.
    pub permissions: serde_json::Value,
}

#[op2(async)]
#[serde]
pub async fn op_edit_guild_command_permissions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCommandPermissions,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let command_id = parse_command_id(&args.command_id)?;
    let route = format!(
        "PUT /guilds/{}/commands/{}/permissions",
        guild_id.get(),
        command_id.get()
    );
    let permissions_payload = args.permissions;
    let permissions = rest
        .execute(guild_id, route, RestRetry::None, move |http| {
            let permissions_payload = permissions_payload.clone();
            async move {
                http.edit_guild_command_permissions(guild_id, command_id, &permissions_payload)
                    .await
            }
        })
        .await?;
    serde_json::to_value(permissions).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments containing only a guild ID.
#[expose_input]
pub struct RawGuildId {
    /// The guild's snowflake ID.
    pub guild_id: String,
}

#[op2(async)]
#[serde]
pub async fn op_get_guild_command_permissions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGetGuildCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let command_id = parse_command_id(&args.command_id)?;
    let route = format!(
        "GET /guilds/{}/commands/{}/permissions",
        guild_id.get(),
        command_id.get()
    );
    let permissions = rest
        .execute(
            guild_id,
            route,
            RestRetry::ReadOnly,
            move |http| async move {
                http.get_guild_command_permissions(guild_id, command_id)
                    .await
            },
        )
        .await?;
    serde_json::to_value(permissions).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[op2(async)]
#[serde]
pub async fn op_get_guild_commands_permissions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildId,
) -> Result<Vec<serde_json::Value>, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let route = format!("GET /guilds/{}/commands/permissions", guild_id.get());
    let permissions = rest
        .execute(
            guild_id,
            route,
            RestRetry::ReadOnly,
            move |http| async move { http.get_guild_commands_permissions(guild_id).await },
        )
        .await?;
    permissions
        .into_iter()
        .map(|perm| serde_json::to_value(perm).map_err(|err| JsErrorBox::generic(err.to_string())))
        .collect()
}

fn parse_guild_id(value: &str) -> Result<GuildId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("guild_id", "invalid snowflake"));
    };
    Ok(GuildId::new(id))
}

fn parse_command_id(value: &str) -> Result<CommandId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("command_id", "invalid snowflake"));
    };
    Ok(CommandId::new(id))
}

fn build_command(cmd: RawSlashCommand) -> Result<CreateCommand<'static>, FloraError> {
    let Some(desc) = cmd.description else {
        return Err(FloraError::invalid_input(
            "command.description",
            "slash command must have a description",
        ));
    };
    let mut builder = CreateCommand::new(cmd.name).description(desc);
    if let Some(options) = cmd.options {
        for opt in options {
            builder = builder.add_option(build_option(opt)?);
        }
    }
    Ok(builder)
}

fn build_option(opt: RawSlashCommandOption) -> Result<CreateCommandOption<'static>, FloraError> {
    let opt_type = match opt.kind.as_deref() {
        Some("integer") => serenity::all::CommandOptionType::Integer,
        Some("number") => serenity::all::CommandOptionType::Number,
        Some("boolean") => serenity::all::CommandOptionType::Boolean,
        Some("subcommand") => serenity::all::CommandOptionType::SubCommand,
        Some("subcommand_group") => serenity::all::CommandOptionType::SubCommandGroup,
        _ => serenity::all::CommandOptionType::String,
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
