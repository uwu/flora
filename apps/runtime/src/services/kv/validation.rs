use color_eyre::eyre::{Result, eyre};

pub(crate) const MAX_VALUE_SIZE: usize = 1024 * 1024;
pub(crate) const DEFAULT_LIST_LIMIT: u32 = 100;
pub(crate) const MAX_LIST_LIMIT: u32 = 1000;
const MAX_KEY_SIZE: usize = 512;
const MAX_STORE_NAME_SIZE: usize = 64;
const MAX_GUILD_ID_SIZE: usize = 32;

pub(crate) fn validate_guild_id(guild_id: &str) -> Result<()> {
    if guild_id.is_empty() {
        return Err(eyre!("guild_id cannot be empty"));
    }
    if guild_id.len() > MAX_GUILD_ID_SIZE {
        return Err(eyre!(
            "guild_id exceeds maximum size of {} characters",
            MAX_GUILD_ID_SIZE
        ));
    }
    if guild_id.contains('/') || guild_id.contains('.') || guild_id.contains("..") {
        return Err(eyre!("guild_id contains invalid characters"));
    }
    Ok(())
}

pub(crate) fn validate_store_name(store_name: &str) -> Result<()> {
    if store_name.is_empty() {
        return Err(eyre!("store_name cannot be empty"));
    }
    if store_name.len() > MAX_STORE_NAME_SIZE {
        return Err(eyre!(
            "store_name exceeds maximum size of {} characters",
            MAX_STORE_NAME_SIZE
        ));
    }
    if store_name.contains('/') || store_name.contains('.') || store_name.contains("..") {
        return Err(eyre!("store_name contains invalid characters"));
    }
    Ok(())
}

pub(crate) fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(eyre!("key cannot be empty"));
    }
    if key.len() > MAX_KEY_SIZE {
        return Err(eyre!(
            "key exceeds maximum size of {} characters",
            MAX_KEY_SIZE
        ));
    }
    if key.contains('\0') {
        return Err(eyre!("key contains null character"));
    }
    Ok(())
}

pub(crate) fn validate_prefix(prefix: &str) -> Result<()> {
    if prefix.contains('\0') {
        return Err(eyre!("prefix contains null character"));
    }
    Ok(())
}
