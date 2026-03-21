use chrono::Utc;
use color_eyre::eyre::{Result, eyre};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DB, DEFAULT_COLUMN_FAMILY_NAME, Direction, IteratorMode,
    Options,
};
use sqlx::{Pool, Postgres};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tracing::{info, warn};

use super::{
    cache::{BoundedCache, MAX_DB_CACHE_SIZE},
    models::{KvStore, KvStoreRow, RawKvKeyInfo, RawKvKeyMetadata, RawKvListKeysResult},
    utils::{copy_dir_all, db_key, to_kv_store},
    validation::{
        DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT, MAX_VALUE_SIZE, validate_guild_id, validate_key,
        validate_prefix, validate_store_name,
    },
};

const METADATA_CF_NAME: &str = "__metadata";

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
            let options = Options::default();
            if let Err(err) = DB::destroy(&options, &db_path) {
                warn!(target: "flora:kv", guild_id, store_name, ?err, "failed to destroy rocksdb files");
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
    ) -> Result<Option<(String, Option<RawKvKeyMetadata>)>> {
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
        db.put(key.as_bytes(), value.as_bytes())?;

        let metadata_cf = self.get_metadata_cf(&db)?;
        if expiration.is_some() || metadata.is_some() {
            let key_metadata = RawKvKeyMetadata {
                expiration,
                metadata,
            };
            let metadata_bytes = serde_json::to_vec(&key_metadata)?;
            db.put_cf(metadata_cf, key.as_bytes(), metadata_bytes)?;
        } else {
            db.delete_cf(metadata_cf, key.as_bytes())?;
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
        let new_expiration = existing.and_then(|m| m.expiration);
        let metadata_cf = self.get_metadata_cf(&db)?;

        if metadata.is_some() || new_expiration.is_some() {
            let key_metadata = RawKvKeyMetadata {
                expiration: new_expiration,
                metadata,
            };
            let metadata_bytes = serde_json::to_vec(&key_metadata)?;
            db.put_cf(metadata_cf, key.as_bytes(), metadata_bytes)?;
        } else {
            db.delete_cf(metadata_cf, key.as_bytes())?;
        }

        Ok(())
    }

    pub async fn delete(&self, guild_id: &str, store_name: &str, key: &str) -> Result<()> {
        validate_key(key)?;
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;
        db.delete(key.as_bytes())?;
        let metadata_cf = self.get_metadata_cf(&db)?;
        db.delete_cf(metadata_cf, key.as_bytes())?;
        Ok(())
    }

    pub async fn list_keys(
        &self,
        guild_id: &str,
        store_name: &str,
        prefix: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<RawKvListKeysResult> {
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
            Some(c) => c.to_string(),
            None => prefix.unwrap_or_default().to_string(),
        };

        let iter = db.iterator(IteratorMode::From(start_key.as_bytes(), Direction::Forward));

        for item in iter {
            let (key_bytes, _) = item?;
            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => continue,
            };

            if let Some(p) = prefix
                && !key.starts_with(p)
            {
                break;
            }

            if let Some(c) = cursor
                && key == c
            {
                continue;
            }

            let metadata = self.get_metadata(&db, &key)?;
            let key_info = RawKvKeyInfo {
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
        let cursor = if list_complete {
            None
        } else {
            keys.last().map(|k| k.name.clone())
        };

        Ok(RawKvListKeysResult {
            keys,
            list_complete,
            cursor,
        })
    }

    pub async fn export_guild(&self, guild_id: &str) -> Result<String> {
        let stores = self.list_stores(guild_id).await?;
        if stores.is_empty() {
            return Err(eyre!("no stores found for guild"));
        }

        let backup_id = Utc::now().timestamp().to_string();
        let backup_dir = self
            .base_path
            .join(guild_id)
            .join("backups")
            .join(&backup_id);
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

    fn get_or_open_db(&self, guild_id: &str, store_name: &str) -> Result<Arc<DB>> {
        let key = db_key(guild_id, store_name);

        if let Ok(cache) = self.db_cache.read()
            && let Some(db) = cache.get(&key)
        {
            return Ok(Arc::clone(db));
        }

        let db_path = self.db_path(guild_id, store_name);
        let db = self.open_db(&db_path)?;
        let db_arc = Arc::new(db);

        if let Ok(mut cache) = self.db_cache.write() {
            cache.insert(key, Arc::clone(&db_arc));
        }

        info!(target: "flora:kv", guild_id, store_name, "opened rocksdb instance");
        Ok(db_arc)
    }

    fn open_db(&self, db_path: &PathBuf) -> Result<DB> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let column_families = vec![
            ColumnFamilyDescriptor::new(DEFAULT_COLUMN_FAMILY_NAME, Options::default()),
            ColumnFamilyDescriptor::new(METADATA_CF_NAME, Options::default()),
        ];

        Ok(DB::open_cf_descriptors(&options, db_path, column_families)?)
    }

    fn get_metadata_cf<'a>(&self, db: &'a DB) -> Result<&'a ColumnFamily> {
        db.cf_handle(METADATA_CF_NAME)
            .ok_or_else(|| eyre!("metadata column family missing"))
    }

    fn get_metadata(&self, db: &DB, key: &str) -> Result<Option<RawKvKeyMetadata>> {
        let metadata_cf = self.get_metadata_cf(db)?;
        match db.get_cf(metadata_cf, key.as_bytes())? {
            Some(bytes) => {
                let metadata: RawKvKeyMetadata = serde_json::from_slice(&bytes)?;
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
