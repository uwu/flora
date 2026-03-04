use color_eyre::eyre::Result;
use std::path::PathBuf;

use super::models::{KvStore, KvStoreRow};

pub(crate) fn db_key(guild_id: &str, store_name: &str) -> String {
    format!("{}:{}", guild_id, store_name)
}

pub(crate) fn to_kv_store(row: KvStoreRow) -> KvStore {
    KvStore {
        id: row.id.to_string(),
        guild_id: row.guild_id,
        store_name: row.store_name,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub(crate) fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
