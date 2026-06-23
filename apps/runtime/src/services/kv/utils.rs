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
