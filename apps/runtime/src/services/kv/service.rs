use chrono::Utc;
use color_eyre::eyre::{Result, eyre};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DB, DEFAULT_COLUMN_FAMILY_NAME, Direction, IteratorMode,
    Options, checkpoint::Checkpoint,
};
use sqlx::{Pool, Postgres};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tracing::{info, warn};
use uuid::Uuid;

use super::{
    cache::{BoundedCache, MAX_DB_CACHE_SIZE},
    models::{KvStore, KvStoreRow, RawKvKeyInfo, RawKvKeyMetadata, RawKvListKeysResult},
    utils::{db_key, to_kv_store},
    validation::{
        DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT, MAX_VALUE_SIZE, validate_guild_id, validate_key,
        validate_prefix, validate_store_name,
    },
};

const METADATA_CF_NAME: &str = "__metadata";
const MAX_ENTRY_BYTES: u64 = 2 * 1024 * 1024;
const MAX_STORE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_GUILD_BYTES: u64 = 500 * 1024 * 1024;
const MAX_STORE_WRITES_PER_MINUTE: usize = 120;
const WRITE_RATE_WINDOW_SECONDS: u64 = 60;
const MIN_GUILD_EXPORT_INTERVAL_SECONDS: u64 = 60;
const MAX_BACKUPS_PER_GUILD: usize = 5;
const RATE_LIMIT_TRACKING_TTL_SECONDS: u64 = 60 * 60;

#[derive(Clone)]
pub struct KvService {
    db: Pool<Postgres>,
    db_cache: Arc<RwLock<BoundedCache>>,
    base_path: PathBuf,
    write_throttle: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
    export_throttle: Arc<Mutex<HashMap<String, Instant>>>,
}

impl KvService {
    pub fn new(db: Pool<Postgres>, base_path: PathBuf) -> Self {
        Self {
            db,
            db_cache: Arc::new(RwLock::new(BoundedCache::new(MAX_DB_CACHE_SIZE))),
            base_path,
            write_throttle: Arc::new(Mutex::new(HashMap::new())),
            export_throttle: Arc::new(Mutex::new(HashMap::new())),
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
        validate_guild_id(guild_id)?;
        validate_store_name(store_name)?;
        self.verify_store_exists(guild_id, store_name).await?;

        let db_key = db_key(guild_id, store_name);
        if let Ok(mut cache) = self.db_cache.write() {
            cache.remove(&db_key);
        }

        let db_path = self.db_path(guild_id, store_name);
        if db_path.exists() {
            let options = Options::default();
            DB::destroy(&options, &db_path).map_err(|err| {
                eyre!("failed to destroy rocksdb files for kv store {guild_id}/{store_name}: {err}")
            })?;
        }

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

        info!(target: "flora:kv", guild_id, store_name, "deleted kv store");
        Ok(())
    }

    pub async fn list_stores(&self, guild_id: &str) -> Result<Vec<KvStore>> {
        validate_guild_id(guild_id)?;

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

        let Some(bytes) = db.get(key.as_bytes())? else {
            return Ok(None);
        };

        let metadata = self.get_metadata(&db, key)?;
        if self.purge_if_expired(&db, key, metadata.as_ref())? {
            return Ok(None);
        }

        let value = String::from_utf8(bytes.to_vec())?;
        Ok(Some(value))
    }

    pub async fn get_with_metadata(
        &self,
        guild_id: &str,
        store_name: &str,
        key: &str,
    ) -> Result<Option<(String, Option<RawKvKeyMetadata>)>> {
        validate_key(key)?;
        self.verify_store_exists(guild_id, store_name).await?;

        let db = self.get_or_open_db(guild_id, store_name)?;

        let value = match db.get(key.as_bytes())? {
            Some(bytes) => String::from_utf8(bytes.to_vec())?,
            None => return Ok(None),
        };

        let metadata = self.get_metadata(&db, key)?;
        if self.purge_if_expired(&db, key, metadata.as_ref())? {
            return Ok(None);
        }

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
        self.enforce_write_rate_limit(&db_key(guild_id, store_name))?;

        self.purge_expired_store(&db)?;
        let new_entry_size =
            self.entry_size_for_write(key, value, expiration, metadata.as_ref())?;
        if new_entry_size > MAX_ENTRY_BYTES {
            return Err(eyre!("kv entry exceeds maximum size of 2MB"));
        }
        self.enforce_write_quota(guild_id, store_name, &db, key, new_entry_size)
            .await?;

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
        if self.purge_if_expired(&db, key, existing.as_ref())? || db.get(key.as_bytes())?.is_none()
        {
            return Err(eyre!("key not found"));
        }

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
        self.delete_key_in_db(&db, key)?;
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
        if limit == 0 {
            return Ok(RawKvListKeysResult {
                keys,
                list_complete: true,
                cursor: None,
            });
        }

        let mut expired_keys = Vec::new();

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
            if Self::is_expired(metadata.as_ref()) {
                expired_keys.push(key);
                continue;
            }

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

        for key in expired_keys {
            self.delete_key_in_db(&db, &key)?;
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

    pub async fn export_guild(&self, guild_id: &str, actor_id: &str) -> Result<String> {
        validate_guild_id(guild_id)?;

        let stores = self.list_stores(guild_id).await?;
        if stores.is_empty() {
            return Err(eyre!("no stores found for guild"));
        }

        self.enforce_rate_limit(
            &self.export_throttle,
            &format!("{guild_id}:{actor_id}"),
            Duration::from_secs(MIN_GUILD_EXPORT_INTERVAL_SECONDS),
            "kv export rate limit exceeded",
        )?;
        self.enforce_rate_limit(
            &self.export_throttle,
            guild_id,
            Duration::from_secs(MIN_GUILD_EXPORT_INTERVAL_SECONDS),
            "kv export rate limit exceeded",
        )?;

        let backup_id = Uuid::new_v4().to_string();
        let backup_root = self.base_path.join(guild_id).join("backups");
        let backup_dir = backup_root.join(&backup_id);
        let temp_backup_dir = backup_root.join(format!(".{backup_id}.tmp"));

        fs::create_dir_all(&backup_root)?;
        self.prune_backups(&backup_root, MAX_BACKUPS_PER_GUILD.saturating_sub(1))?;
        if temp_backup_dir.exists() {
            fs::remove_dir_all(&temp_backup_dir)?;
        }
        fs::create_dir(&temp_backup_dir)?;

        let backup_result = (|| -> Result<()> {
            let mut exported_bytes = 0u64;

            for store in &stores {
                let store_backup_dir = temp_backup_dir.join(&store.store_name);
                let db = self.get_or_open_db(guild_id, &store.store_name)?;

                self.purge_expired_store(&db)?;
                exported_bytes = exported_bytes.saturating_add(self.store_size_bytes(&db)?);
                if exported_bytes > MAX_GUILD_BYTES {
                    return Err(eyre!("kv export exceeds guild storage quota"));
                }

                db.flush()?;

                let checkpoint = Checkpoint::new(db.as_ref())?;
                checkpoint.create_checkpoint(&store_backup_dir)?;

                info!(target: "flora:kv", guild_id, store_name = store.store_name.as_str(), "backed up kv store");
            }

            Ok(())
        })();

        if let Err(err) = backup_result {
            if let Err(cleanup_err) = fs::remove_dir_all(&temp_backup_dir) {
                warn!(target: "flora:kv", guild_id, backup_id, ?cleanup_err, "failed to clean up failed kv export");
            }
            return Err(err);
        }

        fs::rename(&temp_backup_dir, &backup_dir)?;
        self.prune_backups(&backup_root, MAX_BACKUPS_PER_GUILD)?;

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

    fn purge_if_expired(
        &self,
        db: &DB,
        key: &str,
        metadata: Option<&RawKvKeyMetadata>,
    ) -> Result<bool> {
        if !Self::is_expired(metadata) {
            return Ok(false);
        }

        self.delete_key_in_db(db, key)?;
        Ok(true)
    }

    fn purge_expired_store(&self, db: &DB) -> Result<usize> {
        let mut expired_keys = Vec::new();

        for item in db.iterator(IteratorMode::Start) {
            let (key_bytes, _) = item?;
            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(key) => key,
                Err(_) => continue,
            };

            let metadata = self.get_metadata(db, &key)?;
            if Self::is_expired(metadata.as_ref()) {
                expired_keys.push(key);
            }
        }

        let expired_count = expired_keys.len();
        for key in expired_keys {
            self.delete_key_in_db(db, &key)?;
        }

        Ok(expired_count)
    }

    fn is_expired(metadata: Option<&RawKvKeyMetadata>) -> bool {
        metadata
            .and_then(|metadata| metadata.expiration)
            .is_some_and(|expiration| expiration <= Utc::now().timestamp())
    }

    fn delete_key_in_db(&self, db: &DB, key: &str) -> Result<()> {
        db.delete(key.as_bytes())?;
        let metadata_cf = self.get_metadata_cf(db)?;
        db.delete_cf(metadata_cf, key.as_bytes())?;
        Ok(())
    }

    fn entry_size_for_write(
        &self,
        key: &str,
        value: &str,
        expiration: Option<i64>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<u64> {
        let mut size = key.len() as u64 + value.len() as u64;

        if expiration.is_some() || metadata.is_some() {
            let key_metadata = RawKvKeyMetadata {
                expiration,
                metadata: metadata.cloned(),
            };
            let metadata_bytes = serde_json::to_vec(&key_metadata)?;
            size += key.len() as u64 + metadata_bytes.len() as u64;
        }

        Ok(size)
    }

    fn current_entry_size(&self, db: &DB, key: &str) -> Result<u64> {
        let mut size = 0;

        if let Some(value) = db.get(key.as_bytes())? {
            size += key.len() as u64 + value.len() as u64;
        }

        let metadata_cf = self.get_metadata_cf(db)?;
        if let Some(metadata) = db.get_cf(metadata_cf, key.as_bytes())? {
            size += key.len() as u64 + metadata.len() as u64;
        }

        Ok(size)
    }

    fn store_size_bytes(&self, db: &DB) -> Result<u64> {
        let mut size = 0u64;

        for item in db.iterator(IteratorMode::Start) {
            let (key, value) = item?;
            size = size.saturating_add(key.len() as u64 + value.len() as u64);
        }

        let metadata_cf = self.get_metadata_cf(db)?;
        for item in db.iterator_cf(metadata_cf, IteratorMode::Start) {
            let (key, value) = item?;
            size = size.saturating_add(key.len() as u64 + value.len() as u64);
        }

        Ok(size)
    }

    async fn enforce_write_quota(
        &self,
        guild_id: &str,
        store_name: &str,
        db: &DB,
        key: &str,
        new_entry_size: u64,
    ) -> Result<()> {
        let current_store_size = self.store_size_bytes(db)?;
        let existing_entry_size = self.current_entry_size(db, key)?;
        let projected_store_size = current_store_size
            .saturating_sub(existing_entry_size)
            .saturating_add(new_entry_size);

        if projected_store_size > MAX_STORE_BYTES {
            return Err(eyre!("kv store storage quota exceeded"));
        }

        let mut projected_guild_size = projected_store_size;
        for store in self.list_stores(guild_id).await? {
            if store.store_name == store_name {
                continue;
            }

            let store_db = self.get_or_open_db(guild_id, &store.store_name)?;
            self.purge_expired_store(&store_db)?;
            projected_guild_size =
                projected_guild_size.saturating_add(self.store_size_bytes(&store_db)?);

            if projected_guild_size > MAX_GUILD_BYTES {
                return Err(eyre!("kv guild storage quota exceeded"));
            }
        }

        Ok(())
    }

    fn enforce_write_rate_limit(&self, scope: &str) -> Result<()> {
        let now = Instant::now();
        let window = Duration::from_secs(WRITE_RATE_WINDOW_SECONDS);
        let mut throttle = self
            .write_throttle
            .lock()
            .map_err(|_| eyre!("kv rate limiter lock poisoned"))?;

        for events in throttle.values_mut() {
            while events
                .front()
                .is_some_and(|seen| now.saturating_duration_since(*seen) >= window)
            {
                events.pop_front();
            }
        }
        throttle.retain(|_, events| !events.is_empty());

        let events = throttle.entry(scope.to_string()).or_default();
        if events.len() >= MAX_STORE_WRITES_PER_MINUTE {
            return Err(eyre!("kv write rate limit exceeded; retry later"));
        }

        events.push_back(now);
        Ok(())
    }

    fn enforce_rate_limit(
        &self,
        throttle: &Arc<Mutex<HashMap<String, Instant>>>,
        scope: &str,
        interval: Duration,
        message: &str,
    ) -> Result<()> {
        let now = Instant::now();
        let mut last_seen = throttle
            .lock()
            .map_err(|_| eyre!("kv rate limiter lock poisoned"))?;

        last_seen.retain(|_, seen| {
            now.saturating_duration_since(*seen)
                < Duration::from_secs(RATE_LIMIT_TRACKING_TTL_SECONDS)
        });

        if let Some(previous) = last_seen.get(scope) {
            let elapsed = now.saturating_duration_since(*previous);
            if elapsed < interval {
                let retry_after = interval.saturating_sub(elapsed);
                return Err(eyre!("{message}; retry in {}ms", retry_after.as_millis()));
            }
        }

        last_seen.insert(scope.to_string(), now);
        Ok(())
    }

    fn prune_backups(&self, backup_root: &PathBuf, keep: usize) -> Result<()> {
        let mut backups = self.backup_dirs(backup_root)?;
        if backups.len() <= keep {
            return Ok(());
        }

        backups.sort_by(|(left, _), (right, _)| left.cmp(right));
        let remove_count = backups.len().saturating_sub(keep);
        for (_, path) in backups.into_iter().take(remove_count) {
            fs::remove_dir_all(path)?;
        }

        Ok(())
    }

    fn backup_dirs(&self, backup_root: &PathBuf) -> Result<Vec<(Duration, PathBuf)>> {
        if !backup_root.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        for entry in fs::read_dir(backup_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let file_name = entry.file_name();
            if file_name.to_string_lossy().starts_with('.') {
                continue;
            }

            let modified = entry
                .metadata()?
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(SystemTime::UNIX_EPOCH).ok())
                .unwrap_or_default();
            backups.push((modified, entry.path()));
        }

        Ok(backups)
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
