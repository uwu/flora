use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use color_eyre::eyre::{Context, Result};
use hmac::{Hmac, Mac};
use rand::{RngCore, rngs::OsRng};
use serde::Serialize;
use sha2::Sha256;
use sqlx::{FromRow, Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::warn;
use uuid::Uuid;

#[cfg(test)]
use sqlx::postgres::PgPoolOptions;

const PLACEHOLDER_PREFIX: &str = "__FLORA_SECRET__";
pub const THIRDPARTY_DISCORD_TOKEN_MAGIC: &str = "__FLORA_THIRDPARTY_DISCORD_TOKEN__";
const THIRDPARTY_DISCORD_TOKEN_ALLOWED_HOSTS: [&str; 3] =
    ["discord.com", "discord.gg", "gateway.discord.gg"];

#[derive(Clone)]
pub struct SecretService {
    db_pool: Pool<Postgres>,
    key_bytes: [u8; 32],
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretMetadata {
    pub name: String,
    pub allowed_hosts: Vec<String>,
}

#[derive(FromRow)]
struct SecretRow {
    id: Uuid,
    name: String,
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecretRuntimeEntry {
    pub placeholder: String,
    pub value: String,
    pub allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SecretsRuntimeData {
    pub by_name: HashMap<String, SecretRuntimeEntry>,
    pub by_placeholder: HashMap<String, SecretRuntimeEntry>,
}

impl SecretService {
    pub fn new(db_pool: Pool<Postgres>, master_key: String) -> Result<Self> {
        let key_bytes = normalize_key(master_key)?;
        Ok(Self { db_pool, key_bytes })
    }

    pub async fn upsert_secret(
        &self,
        guild_id: &str,
        name: &str,
        value: &str,
        allowed_hosts: Vec<String>,
    ) -> Result<SecretMetadata> {
        let cipher = XChaCha20Poly1305::new(&self.key_bytes.into());
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, value.as_bytes())
            .context("encrypt secret")?;

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO guild_secrets (id, guild_id, name, ciphertext, nonce, allowed_hosts)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (guild_id, name) DO UPDATE
            SET ciphertext = EXCLUDED.ciphertext,
                nonce = EXCLUDED.nonce,
                allowed_hosts = EXCLUDED.allowed_hosts,
                updated_at = NOW()
            "#,
        )
        .bind(id)
        .bind(guild_id)
        .bind(name)
        .bind(&ciphertext)
        .bind(&nonce_bytes.to_vec())
        .bind(&allowed_hosts)
        .execute(&self.db_pool)
        .await
        .context("store secret")?;

        Ok(SecretMetadata {
            name: name.to_string(),
            allowed_hosts,
        })
    }

    pub async fn delete_secret(&self, guild_id: &str, name: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM guild_secrets
            WHERE guild_id = $1 AND name = $2
            "#,
        )
        .bind(guild_id)
        .bind(name)
        .execute(&self.db_pool)
        .await
        .context("delete secret")?;
        Ok(())
    }

    pub async fn list_metadata(&self, guild_id: &str) -> Result<Vec<SecretMetadata>> {
        let rows = sqlx::query_as::<_, SecretRow>(
            r#"
            SELECT id, name, ciphertext, nonce, allowed_hosts
            FROM guild_secrets
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.db_pool)
        .await
        .context("list secrets")?;

        Ok(rows
            .into_iter()
            .map(|row| SecretMetadata {
                name: row.name,
                allowed_hosts: row.allowed_hosts,
            })
            .collect())
    }

    pub async fn load_runtime(&self, guild_id: &str) -> Result<Arc<SecretsRuntimeData>> {
        let rows = sqlx::query_as::<_, SecretRow>(
            r#"
            SELECT id, name, ciphertext, nonce, allowed_hosts
            FROM guild_secrets
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.db_pool)
        .await
        .context("load secrets for runtime")?;

        let cipher = XChaCha20Poly1305::new(&self.key_bytes.into());
        let mut by_name = HashMap::new();
        let mut by_placeholder = HashMap::new();

        for row in rows {
            let Ok(nonce_bytes) = <[u8; 24]>::try_from(row.nonce.as_slice()) else {
                warn!(target: "flora:secrets", guild_id, name = row.name, "invalid nonce length");
                continue;
            };
            let nonce = XNonce::from_slice(&nonce_bytes);
            let Ok(plaintext) = cipher.decrypt(nonce, row.ciphertext.as_ref()) else {
                warn!(target: "flora:secrets", guild_id, name = row.name, "failed to decrypt secret");
                continue;
            };
            let value = match String::from_utf8(plaintext) {
                Ok(v) => v,
                Err(_) => {
                    warn!(target: "flora:secrets", guild_id, name = row.name, "secret not utf8");
                    continue;
                }
            };
            let placeholder = build_placeholder(row.id, &self.key_bytes);
            let entry = SecretRuntimeEntry {
                placeholder: placeholder.clone(),
                value,
                allowed_hosts: row.allowed_hosts.clone(),
            };
            by_placeholder.insert(placeholder, entry.clone());
            by_name.insert(row.name, entry);
        }

        // Flora-only magic marker used by discord.js polyfills.
        let third_party_entry = SecretRuntimeEntry {
            placeholder: THIRDPARTY_DISCORD_TOKEN_MAGIC.to_string(),
            value: THIRDPARTY_DISCORD_TOKEN_MAGIC.to_string(),
            allowed_hosts: THIRDPARTY_DISCORD_TOKEN_ALLOWED_HOSTS
                .iter()
                .map(|host| host.to_string())
                .collect(),
        };
        by_placeholder.insert(
            THIRDPARTY_DISCORD_TOKEN_MAGIC.to_string(),
            third_party_entry.clone(),
        );
        by_name.insert(
            THIRDPARTY_DISCORD_TOKEN_MAGIC.to_string(),
            third_party_entry,
        );

        Ok(Arc::new(SecretsRuntimeData {
            by_name,
            by_placeholder,
        }))
    }
}

pub fn build_placeholder(id: Uuid, key_bytes: &[u8; 32]) -> String {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key_bytes).expect("hmac key");
    mac.update(id.as_bytes());
    let tag = mac.finalize().into_bytes();
    let short = hex::encode(&tag)[0..12].to_string();
    format!("{PLACEHOLDER_PREFIX}:{id}:{short}")
}

fn normalize_key(master_key: String) -> Result<[u8; 32]> {
    let bytes = if master_key.len() == 64 && master_key.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut out = [0u8; 32];
        hex::decode_to_slice(master_key, &mut out as &mut [u8]).context("decode hex master key")?;
        out
    } else {
        let raw = master_key.as_bytes();
        if raw.len() != 32 {
            color_eyre::eyre::bail!("SECRETS_MASTER_KEY must be 32 bytes (got {})", raw.len());
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(raw);
        out
    };
    Ok(bytes)
}

impl SecretsRuntimeData {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn placeholder_for(&self, name: &str) -> Option<String> {
        if name == THIRDPARTY_DISCORD_TOKEN_MAGIC {
            return Some(THIRDPARTY_DISCORD_TOKEN_MAGIC.to_string());
        }

        self.by_name.get(name).map(|s| s.placeholder.clone())
    }
}

#[cfg(test)]
impl SecretService {
    pub async fn new_for_tests(database_url: &str) -> Self {
        let db_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .expect("create test pg pool");
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS guild_secrets (
                id UUID PRIMARY KEY,
                guild_id TEXT NOT NULL,
                name TEXT NOT NULL,
                ciphertext BYTEA NOT NULL,
                nonce BYTEA NOT NULL,
                allowed_hosts TEXT[] NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (guild_id, name)
            );
            "#,
        )
        .execute(&db_pool)
        .await
        .expect("create secrets schema");
        Self {
            db_pool,
            key_bytes: [0u8; 32],
        }
    }

    pub async fn load_runtime_for_tests(&self, _guild_id: &str) -> Result<Arc<SecretsRuntimeData>> {
        Ok(SecretsRuntimeData::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::{SecretsRuntimeData, THIRDPARTY_DISCORD_TOKEN_MAGIC};

    #[test]
    fn placeholder_for_returns_third_party_discord_magic() {
        let data = SecretsRuntimeData::default();
        assert_eq!(
            data.placeholder_for(THIRDPARTY_DISCORD_TOKEN_MAGIC),
            Some(THIRDPARTY_DISCORD_TOKEN_MAGIC.to_string())
        );
    }
}
