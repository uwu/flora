use std::{cell::RefCell, rc::Rc, sync::Arc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    builder::{CreateCommand, CreateCommandOption},
    http::Http,
    model::id::{CommandId, GuildId},
};

use super::interaction::{RawSlashCommand, RawSlashCommandOption};

/// Arguments for bulk-upserting global application commands.
#[expose_input]
pub(crate) struct RawUpsertGlobalCommands {
    /// The commands to register globally.
    pub commands: Vec<RawSlashCommand>,
}

#[op2(async)]
#[serde]
pub async fn op_upsert_global_commands(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawUpsertGlobalCommands,
) -> Result<Vec<serde_json::Value>, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let commands = build_commands(args.commands)?;
    let created = http
        .create_global_commands(&commands)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    created
        .into_iter()
        .map(|cmd| serde_json::to_value(cmd).map_err(|err| JsErrorBox::generic(err.to_string())))
        .collect()
}

/// Arguments for creating a single global application command.
#[expose_input]
pub(crate) struct RawCreateGlobalCommand {
    /// The command to create.
    pub command: RawSlashCommand,
}

#[op2(async)]
#[serde]
pub async fn op_create_global_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateGlobalCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let command = build_command(args.command)?;
    let created = http
        .create_global_command(&command)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(created).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing a global application command.
#[expose_input]
pub(crate) struct RawEditGlobalCommand {
    /// The command's snowflake ID.
    pub command_id: String,
    /// Updated command data.
    pub command: RawSlashCommand,
}

#[op2(async)]
#[serde]
pub async fn op_edit_global_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditGlobalCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let command_id = parse_command_id(&args.command_id)?;
    let command = build_command(args.command)?;
    let updated = http
        .edit_global_command(command_id, &command)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(updated).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for deleting a global application command.
#[expose_input]
pub(crate) struct RawDeleteGlobalCommand {
    /// The command's snowflake ID.
    pub command_id: String,
}

#[op2(async)]
pub async fn op_delete_global_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteGlobalCommand,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let command_id = parse_command_id(&args.command_id)?;
    http.delete_global_command(command_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_get_global_commands(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<serde_json::Value>, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let commands = http
        .get_global_commands()
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    commands
        .into_iter()
        .map(|cmd| serde_json::to_value(cmd).map_err(|err| JsErrorBox::generic(err.to_string())))
        .collect()
}

/// Arguments for fetching a global application command.
#[expose_input]
pub(crate) struct RawGetGlobalCommand {
    /// The command's snowflake ID.
    pub command_id: String,
}

#[op2(async)]
#[serde]
pub async fn op_get_global_command(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGetGlobalCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let command_id = parse_command_id(&args.command_id)?;
    let command = http
        .get_global_command(command_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(command).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for creating a guild application command.
#[expose_input]
pub(crate) struct RawCreateGuildCommand {
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
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let command = build_command(args.command)?;
    let created = http
        .create_guild_command(guild_id, &command)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(created).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing a guild application command.
#[expose_input]
pub(crate) struct RawEditGuildCommand {
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
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let command_id = parse_command_id(&args.command_id)?;
    let command = build_command(args.command)?;
    let updated = http
        .edit_guild_command(guild_id, command_id, &command)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(updated).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for deleting a guild application command.
#[expose_input]
pub(crate) struct RawDeleteGuildCommand {
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
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let command_id = parse_command_id(&args.command_id)?;
    http.delete_guild_command(guild_id, command_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

/// Arguments for fetching a guild application command.
#[expose_input]
pub(crate) struct RawGetGuildCommand {
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
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let commands = http
        .get_guild_commands(guild_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let command_id = parse_command_id(&args.command_id)?;
    let command = http
        .get_guild_command(guild_id, command_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(command).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing guild command permissions.
#[expose_input]
pub(crate) struct RawCommandPermissions {
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
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let command_id = parse_command_id(&args.command_id)?;
    let permissions = http
        .edit_guild_command_permissions(guild_id, command_id, &args.permissions)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(permissions).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments containing only a guild ID.
#[expose_input]
pub(crate) struct RawGuildId {
    /// The guild's snowflake ID.
    pub guild_id: String,
}

#[op2(async)]
#[serde]
pub async fn op_get_guild_command_permissions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGetGuildCommand,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let command_id = parse_command_id(&args.command_id)?;
    let permissions = http
        .get_guild_command_permissions(guild_id, command_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(permissions).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[op2(async)]
#[serde]
pub async fn op_get_guild_commands_permissions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildId,
) -> Result<Vec<serde_json::Value>, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let permissions = http
        .get_guild_commands_permissions(guild_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    permissions
        .into_iter()
        .map(|perm| serde_json::to_value(perm).map_err(|err| JsErrorBox::generic(err.to_string())))
        .collect()
}

fn parse_guild_id(value: &str) -> Result<GuildId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(GuildId::new)
        .map_err(|_| JsErrorBox::generic("Invalid guild id"))
}

fn parse_command_id(value: &str) -> Result<CommandId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(CommandId::new)
        .map_err(|_| JsErrorBox::generic("Invalid command id"))
}

fn build_commands(
    commands: Vec<RawSlashCommand>,
) -> Result<Vec<CreateCommand<'static>>, JsErrorBox> {
    commands.into_iter().map(build_command).collect()
}

fn build_command(cmd: RawSlashCommand) -> Result<CreateCommand<'static>, JsErrorBox> {
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
}

fn build_option(
    opt: RawSlashCommandOption,
) -> Result<CreateCommandOption<'static>, JsErrorBox> {
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
