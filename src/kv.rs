use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use sled::Db;
use sqlx::{FromRow, Pool, Postgres};
use tracing::{info, warn};
use utoipa::ToSchema;

const MAX_VALUE_SIZE: usize = 1024 * 1024; // 1MB

/// Metadata for a KV store.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KvStore {
    pub id: String,
    pub guild_id: String,
    pub store_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct KvStoreRow {
    id: sqlx::types::Uuid,
    guild_id: String,
    store_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct KvService {
    db: Pool<Postgres>,
    db_cache: Arc<RwLock<HashMap<String, Arc<Db>>>>,
    base_path: PathBuf,
}

impl KvService {
    pub fn new(db: Pool<Postgres>, base_path: PathBuf) -> Self {
        Self { db, db_cache: Arc::new(RwLock::new(HashMap::new())), base_path }
    }

    /// Create a new KV store for a guild.
    pub async fn create_store(&self, guild_id: String, store_name: String) -> Result<KvStore> {
        let row = sqlx::query_as::<_, KvStoreRow>(
            r#"
            INSERT INTO kv_stores (guild_id, store_name)
            VALUES ($1, $2)
            RETURNING id, guild_id, store_name, created_at, updated_at
            "#,
        )
        .bind(&guild_id)
        .bind(&store_name)
        .fetch_one(&self.db)
        .await?;

        info!(target: "flora:kv", guild_id, store_name, "created kv store");
        Ok(to_kv_store(row))
    }

    /// Delete a KV store and its data.
    pub async fn delete_store(&self, guild_id: &str, store_name: &str) -> Result<()> {
        // Remove from database
        let result = sqlx::query(
            r#"
            DELETE FROM kv_stores
            WHERE guild_id = $1 AND store_name = $2
            "#,
        )
        .bind(guild_id)
        .bind(store_name)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(eyre!("store not found"));
        }

        // Close DB instance if open
        let db_key = db_key(guild_id, store_name);
        if let Ok(mut cache) = self.db_cache.write() {
            cache.remove(&db_key);
        }

        // Remove sled files (best effort)
        let db_path = self.db_path(guild_id, store_name);
        if db_path.exists() {
            if let Err(err) = std::fs::remove_dir_all(&db_path) {
                warn!(target: "flora:kv", guild_id, store_name, ?err, "failed to remove sled files");
            }
        }

        info!(target: "flora:kv", guild_id, store_name, "deleted kv store");
        Ok(())
    }

    /// List all KV stores for a guild.
    pub async fn list_stores(&self, guild_id: &str) -> Result<Vec<KvStore>> {
        let rows = sqlx::query_as::<_, KvStoreRow>(
            r#"
            SELECT id, guild_id, store_name, created_at, updated_at
            FROM kv_stores
            WHERE guild_id = $1
            ORDER BY store_name
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(to_kv_store).collect())
    }

    /// Get a value from a KV store.
    pub async fn get(&self, guild_id: &str, store_name: &str, key: &str) -> Result<Option<String>> {
        // Verify store exists
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;
        match db.get(key.as_bytes())? {
            Some(bytes) => {
                let value = String::from_utf8(bytes.to_vec())?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a value in a KV store.
    pub async fn set(
        &self,
        guild_id: &str,
        store_name: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        // Verify store exists
        self.verify_store_exists(guild_id, store_name).await?;

        // Check value size
        if value.len() > MAX_VALUE_SIZE {
            return Err(eyre!("value exceeds maximum size of 1MB"));
        }

        let db = self.get_or_open_db(guild_id, store_name)?;
        db.insert(key.as_bytes(), value.as_bytes())?;
        Ok(())
    }

    /// Delete a key from a KV store.
    pub async fn delete(&self, guild_id: &str, store_name: &str, key: &str) -> Result<()> {
        // Verify store exists
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;
        db.remove(key.as_bytes())?;
        Ok(())
    }

    /// List all keys in a KV store, optionally filtered by prefix.
    pub async fn list_keys(
        &self,
        guild_id: &str,
        store_name: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<String>> {
        // Verify store exists
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;
        let mut keys = Vec::new();

        for item in db.iter() {
            let (key_bytes, _) = item?;
            if let Ok(key) = String::from_utf8(key_bytes.to_vec()) {
                if let Some(p) = prefix {
                    if key.starts_with(p) {
                        keys.push(key);
                    }
                } else {
                    keys.push(key);
                }
            }
        }

        Ok(keys)
    }

    /// Export all KV stores for a guild using sled export.
    pub async fn export_guild(&self, guild_id: &str) -> Result<PathBuf> {
        let stores = self.list_stores(guild_id).await?;
        if stores.is_empty() {
            return Err(eyre!("no stores found for guild"));
        }

        let backup_dir =
            self.base_path.join(guild_id).join("backups").join(Utc::now().timestamp().to_string());
        std::fs::create_dir_all(&backup_dir)?;

        // Export each store by copying the database directory
        for store in stores {
            let db_path = self.db_path(guild_id, &store.store_name);
            let store_backup_dir = backup_dir.join(&store.store_name);

            // Open or get existing DB to ensure it's flushed
            let db = self.get_or_open_db(guild_id, &store.store_name)?;
            db.flush()?;

            // Copy the database directory
            if db_path.exists() {
                copy_dir_all(&db_path, &store_backup_dir)?;
            }

            info!(target: "flora:kv", guild_id, store_name = store.store_name, "backed up kv store");
        }

        info!(target: "flora:kv", guild_id, backup_path = ?backup_dir, "exported guild kv stores");
        Ok(backup_dir)
    }

    fn get_or_open_db(&self, guild_id: &str, store_name: &str) -> Result<Arc<Db>> {
        let key = db_key(guild_id, store_name);

        // Try to get from cache first
        if let Ok(cache) = self.db_cache.read() {
            if let Some(db) = cache.get(&key) {
                return Ok(Arc::clone(db));
            }
        }

        // Open new DB instance
        let db_path = self.db_path(guild_id, store_name);
        let db = sled::open(&db_path)?;
        let db_arc = Arc::new(db);

        // Cache it
        if let Ok(mut cache) = self.db_cache.write() {
            cache.insert(key, Arc::clone(&db_arc));
        }

        info!(target: "flora:kv", guild_id, store_name, "opened sled instance");
        Ok(db_arc)
    }

    async fn verify_store_exists(&self, guild_id: &str, store_name: &str) -> Result<()> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM kv_stores
                WHERE guild_id = $1 AND store_name = $2
            )
            "#,
        )
        .bind(guild_id)
        .bind(store_name)
        .fetch_one(&self.db)
        .await?;

        if !exists {
            return Err(eyre!("store not found"));
        }

        Ok(())
    }

    fn db_path(&self, guild_id: &str, store_name: &str) -> PathBuf {
        self.base_path.join(guild_id).join(store_name)
    }
}

fn db_key(guild_id: &str, store_name: &str) -> String {
    format!("{}:{}", guild_id, store_name)
}

fn to_kv_store(row: KvStoreRow) -> KvStore {
    KvStore {
        id: row.id.to_string(),
        guild_id: row.guild_id,
        store_name: row.store_name,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
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
