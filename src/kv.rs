use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use sled::Db;
use sqlx::{FromRow, Pool, Postgres};
use tracing::{info, warn};
use ts_rs::TS;
use utoipa::ToSchema;

const MAX_VALUE_SIZE: usize = 1024 * 1024;
const DEFAULT_LIST_LIMIT: u32 = 100;
const MAX_LIST_LIMIT: u32 = 1000;
const METADATA_TREE_NAME: &str = "__metadata";
const MAX_KEY_SIZE: usize = 512;
const MAX_STORE_NAME_SIZE: usize = 64;
const MAX_GUILD_ID_SIZE: usize = 32;
const MAX_DB_CACHE_SIZE: usize = 64;

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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "sdk/src/generated/")]
pub struct KvKeyMetadata {
    pub expiration: Option<i64>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "sdk/src/generated/")]
pub struct KvKeyInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "sdk/src/generated/")]
pub struct ListKeysResult {
    pub keys: Vec<KvKeyInfo>,
    pub list_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

struct BoundedCache {
    map: HashMap<String, Arc<Db>>,
    order: VecDeque<String>,
    capacity: usize,
}

impl BoundedCache {
    fn new(capacity: usize) -> Self {
        Self { map: HashMap::new(), order: VecDeque::new(), capacity }
    }

    fn get(&self, key: &str) -> Option<&Arc<Db>> {
        self.map.get(key)
    }

    fn insert(&mut self, key: String, db: Arc<Db>) {
        if self.map.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_back() {
                self.map.remove(&oldest);
            }
        }
        self.order.push_front(key.clone());
        self.map.insert(key, db);
    }

    fn remove(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.map.remove(key);
    }
}

#[derive(Clone)]
pub struct KvService {
    db: Pool<Postgres>,
    db_cache: Arc<RwLock<BoundedCache>>,
    base_path: PathBuf,
}

impl KvService {
    pub fn new(db: Pool<Postgres>, base_path: PathBuf) -> Self {
        Self {
            db,
            db_cache: Arc::new(RwLock::new(BoundedCache::new(MAX_DB_CACHE_SIZE))),
            base_path,
        }
    }

    pub async fn create_store(&self, guild_id: String, store_name: String) -> Result<KvStore> {
        validate_guild_id(&guild_id)?;
        validate_store_name(&store_name)?;

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

    pub async fn delete_store(&self, guild_id: &str, store_name: &str) -> Result<()> {
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

        let db_key = db_key(guild_id, store_name);
        if let Ok(mut cache) = self.db_cache.write() {
            cache.remove(&db_key);
        }

        let db_path = self.db_path(guild_id, store_name);
        if db_path.exists() {
            if let Err(err) = std::fs::remove_dir_all(&db_path) {
                warn!(target: "flora:kv", guild_id, store_name, ?err, "failed to remove sled files");
            }
        }

        info!(target: "flora:kv", guild_id, store_name, "deleted kv store");
        Ok(())
    }

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

    pub async fn get(&self, guild_id: &str, store_name: &str, key: &str) -> Result<Option<String>> {
        validate_key(key)?;
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

    pub async fn get_with_metadata(
        &self,
        guild_id: &str,
        store_name: &str,
        key: &str,
    ) -> Result<Option<(String, Option<KvKeyMetadata>)>> {
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;

        let value = match db.get(key.as_bytes())? {
            Some(bytes) => String::from_utf8(bytes.to_vec())?,
            None => return Ok(None),
        };

        let metadata = self.get_metadata(&db, key)?;
        Ok(Some((value, metadata)))
    }

    pub async fn set(
        &self,
        guild_id: &str,
        store_name: &str,
        key: &str,
        value: &str,
        expiration: Option<i64>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        validate_key(key)?;
        self.verify_store_exists(guild_id, store_name).await?;

        if value.len() > MAX_VALUE_SIZE {
            return Err(eyre!("value exceeds maximum size of 1MB"));
        }

        let db = self.get_or_open_db(guild_id, store_name)?;
        db.insert(key.as_bytes(), value.as_bytes())?;

        if expiration.is_some() || metadata.is_some() {
            let key_metadata = KvKeyMetadata { expiration, metadata };
            let metadata_bytes = serde_json::to_vec(&key_metadata)?;
            self.get_metadata_tree(&db)?.insert(key.as_bytes(), metadata_bytes)?;
        } else {
            self.get_metadata_tree(&db)?.remove(key.as_bytes())?;
        }

        Ok(())
    }

    pub async fn update_metadata(
        &self,
        guild_id: &str,
        store_name: &str,
        key: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        validate_key(key)?;
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;

        let existing = self.get_metadata(&db, key)?;
        let new_expiration = existing.map(|m| m.expiration).flatten();

        if metadata.is_some() || new_expiration.is_some() {
            let key_metadata = KvKeyMetadata { expiration: new_expiration, metadata };
            let metadata_bytes = serde_json::to_vec(&key_metadata)?;
            self.get_metadata_tree(&db)?.insert(key.as_bytes(), metadata_bytes)?;
        } else {
            self.get_metadata_tree(&db)?.remove(key.as_bytes())?;
        }

        Ok(())
    }

    pub async fn delete(&self, guild_id: &str, store_name: &str, key: &str) -> Result<()> {
        validate_key(key)?;
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;
        db.remove(key.as_bytes())?;
        self.get_metadata_tree(&db)?.remove(key.as_bytes())?;
        Ok(())
    }

    pub async fn list_keys(
        &self,
        guild_id: &str,
        store_name: &str,
        prefix: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<ListKeysResult> {
        if let Some(p) = prefix {
            validate_prefix(p)?;
        }
        if let Some(c) = cursor {
            validate_key(c)?;
        }
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;
        let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT).min(MAX_LIST_LIMIT);
        let mut keys = Vec::with_capacity(limit as usize);

        let start_key = match cursor {
            Some(c) => {
                if prefix.is_some() {
                    format!("{}{}", prefix.unwrap(), c)
                } else {
                    c.to_string()
                }
            }
            None => prefix.unwrap_or("").to_string(),
        };

        let start_bytes = start_key.as_bytes();
        let iter = db.range(start_bytes..);

        for item in iter {
            let (key_bytes, _) = item?;
            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => continue,
            };

            if let Some(p) = prefix {
                if !key.starts_with(p) {
                    break;
                }
                if let Some(c) = cursor {
                    if key == format!("{}{}", p, c) {
                        continue;
                    }
                }
            }

            let metadata = self.get_metadata(&db, &key)?;
            let key_info = KvKeyInfo {
                name: key.clone(),
                expiration: metadata.as_ref().and_then(|m| m.expiration),
                metadata: metadata.as_ref().and_then(|m| m.metadata.clone()),
            };
            keys.push(key_info);

            if keys.len() >= limit as usize {
                break;
            }
        }

        let list_complete = keys.len() < limit as usize;
        let cursor = if list_complete { None } else { keys.last().map(|k| k.name.clone()) };

        Ok(ListKeysResult { keys, list_complete, cursor })
    }

    pub async fn export_guild(&self, guild_id: &str) -> Result<String> {
        let stores = self.list_stores(guild_id).await?;
        if stores.is_empty() {
            return Err(eyre!("no stores found for guild"));
        }

        let backup_id = Utc::now().timestamp().to_string();
        let backup_dir = self.base_path.join(guild_id).join("backups").join(&backup_id);
        std::fs::create_dir_all(&backup_dir)?;

        for store in stores {
            let db_path = self.db_path(guild_id, &store.store_name);
            let store_backup_dir = backup_dir.join(&store.store_name);

            let db = self.get_or_open_db(guild_id, &store.store_name)?;
            db.flush()?;

            if db_path.exists() {
                copy_dir_all(&db_path, &store_backup_dir)?;
            }

            info!(target: "flora:kv", guild_id, store_name = store.store_name, "backed up kv store");
        }

        info!(target: "flora:kv", guild_id, backup_id, "exported guild kv stores");
        Ok(backup_id)
    }

    fn get_or_open_db(&self, guild_id: &str, store_name: &str) -> Result<Arc<Db>> {
        let key = db_key(guild_id, store_name);

        if let Ok(cache) = self.db_cache.read() {
            if let Some(db) = cache.get(&key) {
                return Ok(Arc::clone(db));
            }
        }

        let db_path = self.db_path(guild_id, store_name);
        let db = sled::open(&db_path)?;
        let db_arc = Arc::new(db);

        if let Ok(mut cache) = self.db_cache.write() {
            cache.insert(key, Arc::clone(&db_arc));
        }

        info!(target: "flora:kv", guild_id, store_name, "opened sled instance");
        Ok(db_arc)
    }

    fn get_metadata_tree(&self, db: &Db) -> Result<sled::Tree> {
        Ok(db.open_tree(METADATA_TREE_NAME)?)
    }

    fn get_metadata(&self, db: &Db, key: &str) -> Result<Option<KvKeyMetadata>> {
        let tree = self.get_metadata_tree(db)?;
        match tree.get(key.as_bytes())? {
            Some(bytes) => {
                let metadata: KvKeyMetadata = serde_json::from_slice(&bytes)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
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

fn validate_guild_id(guild_id: &str) -> Result<()> {
    if guild_id.is_empty() {
        return Err(eyre!("guild_id cannot be empty"));
    }
    if guild_id.len() > MAX_GUILD_ID_SIZE {
        return Err(eyre!("guild_id exceeds maximum size of {} characters", MAX_GUILD_ID_SIZE));
    }
    if guild_id.contains('/') || guild_id.contains('.') || guild_id.contains("..") {
        return Err(eyre!("guild_id contains invalid characters"));
    }
    Ok(())
}

fn validate_store_name(store_name: &str) -> Result<()> {
    if store_name.is_empty() {
        return Err(eyre!("store_name cannot be empty"));
    }
    if store_name.len() > MAX_STORE_NAME_SIZE {
        return Err(eyre!("store_name exceeds maximum size of {} characters", MAX_STORE_NAME_SIZE));
    }
    if store_name.contains('/') || store_name.contains('.') || store_name.contains("..") {
        return Err(eyre!("store_name contains invalid characters"));
    }
    Ok(())
}

fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(eyre!("key cannot be empty"));
    }
    if key.len() > MAX_KEY_SIZE {
        return Err(eyre!("key exceeds maximum size of {} characters", MAX_KEY_SIZE));
    }
    if key.contains('\0') {
        return Err(eyre!("key contains null character"));
    }
    Ok(())
}

fn validate_prefix(prefix: &str) -> Result<()> {
    if prefix.contains('\0') {
        return Err(eyre!("prefix contains null character"));
    }
    Ok(())
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
