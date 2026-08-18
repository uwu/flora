use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, Result, eyre};
use serde::{Deserialize, Serialize};
use serenity::{all::Token, http::Http};
use sqlx::{FromRow, Pool, Postgres};
use utoipa::ToSchema;
use uuid::Uuid;

pub const CUSTOM_BOT_DEPLOYMENT_PREFIX: &str = "__flora_custom_bot__:";

#[derive(Debug, thiserror::Error)]
#[error("account already owns a User Bot")]
pub struct UserBotLimitReached;

pub fn custom_bot_deployment_id(bot_id: Uuid) -> String {
    format!("{CUSTOM_BOT_DEPLOYMENT_PREFIX}{bot_id}")
}

pub fn custom_bot_id_from_deployment_id(scope_id: &str) -> Option<Uuid> {
    scope_id
        .strip_prefix(CUSTOM_BOT_DEPLOYMENT_PREFIX)
        .and_then(|value| Uuid::parse_str(value).ok())
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomBot {
    pub id: Uuid,
    pub owner_user_id: String,
    pub label: Option<String>,
    pub bot_user_id: String,
    pub bot_username: String,
    pub application_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CustomBotWithToken {
    pub bot: CustomBot,
    pub token: String,
}

#[derive(Debug, FromRow)]
struct CustomBotRow {
    id: Uuid,
    owner_user_id: String,
    label: Option<String>,
    bot_user_id: String,
    bot_username: String,
    application_id: String,
    token_ciphertext: Vec<u8>,
    token_nonce: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CustomBotService {
    db: Pool<Postgres>,
    key: [u8; 32],
}

impl CustomBotService {
    pub fn new(db: Pool<Postgres>, master_key: String) -> Result<Self> {
        Ok(Self {
            db,
            key: normalize_key(&master_key)?,
        })
    }

    pub async fn create(
        &self,
        owner_user_id: &str,
        label: Option<String>,
        raw_token: &str,
    ) -> Result<CustomBot> {
        if self.get_for_owner(owner_user_id).await?.is_some() {
            return Err(UserBotLimitReached.into());
        }
        let token_text = raw_token.trim();
        if token_text.is_empty() {
            return Err(eyre!("bot token cannot be empty"));
        }
        let token: Token = token_text
            .parse()
            .map_err(|err: serenity::secrets::TokenError| eyre!(err))
            .context("invalid bot token")?;
        let http = Http::new(token);
        let user = http
            .get_current_user()
            .await
            .context("failed to authenticate bot token")?;
        let application = http
            .get_current_application_info()
            .await
            .context("failed to load bot application")?;
        let (ciphertext, nonce) = self.encrypt(token_text)?;

        let row = sqlx::query_as::<_, CustomBotRow>(
            r#"
            INSERT INTO custom_bots (
                owner_user_id, label, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, owner_user_id, label, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
            "#,
        )
        .bind(owner_user_id)
        .bind(label.filter(|value| !value.trim().is_empty()))
        .bind(user.id.get().to_string())
        .bind(user.name.to_string())
        .bind(application.id.get().to_string())
        .bind(ciphertext)
        .bind(nonce)
        .fetch_one(&self.db)
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(database_err)
                if database_err.code().as_deref() == Some("23505")
                    && database_err.constraint() == Some("custom_bots_owner_user_id_unique") =>
            {
                UserBotLimitReached.into()
            }
            other => eyre!(other).wrap_err("store custom bot"),
        })?;
        Ok(to_custom_bot(row))
    }

    pub async fn get_for_owner(&self, owner_user_id: &str) -> Result<Option<CustomBot>> {
        let row = sqlx::query_as::<_, CustomBotRow>(
            r#"
            SELECT id, owner_user_id, label, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
            FROM custom_bots
            WHERE owner_user_id = $1
            "#,
        )
        .bind(owner_user_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row.map(to_custom_bot))
    }

    pub async fn list_for_owner(&self, owner_user_id: &str) -> Result<Vec<CustomBot>> {
        let rows = sqlx::query_as::<_, CustomBotRow>(
            r#"
            SELECT id, owner_user_id, label, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
            FROM custom_bots
            WHERE owner_user_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
        )
        .bind(owner_user_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows.into_iter().map(to_custom_bot).collect())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<CustomBot>> {
        let row = self.get_row(id).await?;
        Ok(row.map(to_custom_bot))
    }

    pub async fn get_with_token(&self, id: Uuid) -> Result<Option<CustomBotWithToken>> {
        let Some(row) = self.get_row(id).await? else {
            return Ok(None);
        };
        let token = self.decrypt(&row.token_ciphertext, &row.token_nonce)?;
        Ok(Some(CustomBotWithToken {
            bot: to_custom_bot(row),
            token,
        }))
    }

    pub async fn list_with_tokens(&self) -> Result<Vec<CustomBotWithToken>> {
        let rows = sqlx::query_as::<_, CustomBotRow>(
            r#"
            SELECT id, owner_user_id, label, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
            FROM custom_bots
            ORDER BY created_at, id
            "#,
        )
        .fetch_all(&self.db)
        .await?;
        rows.into_iter()
            .map(|row| {
                let token = self.decrypt(&row.token_ciphertext, &row.token_nonce)?;
                Ok(CustomBotWithToken {
                    bot: to_custom_bot(row),
                    token,
                })
            })
            .collect()
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM custom_bots WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_row(&self, id: Uuid) -> Result<Option<CustomBotRow>> {
        sqlx::query_as::<_, CustomBotRow>(
            r#"
            SELECT id, owner_user_id, label, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
            FROM custom_bots
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await
        .map_err(Into::into)
    }

    fn encrypt(&self, value: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        let cipher = XChaCha20Poly1305::new(&self.key.into());
        let mut nonce = [0u8; 24];
        rand::fill(&mut nonce);
        let cipher_nonce = XNonce::from(nonce);
        let ciphertext = cipher
            .encrypt(&cipher_nonce, value.as_bytes())
            .context("encrypt custom bot token")?;
        Ok((ciphertext, nonce.to_vec()))
    }

    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String> {
        let nonce: [u8; 24] = nonce
            .try_into()
            .map_err(|_| eyre!("invalid custom bot token nonce"))?;
        let nonce = XNonce::from(nonce);
        let cipher = XChaCha20Poly1305::new(&self.key.into());
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .context("decrypt custom bot token")?;
        String::from_utf8(plaintext).context("custom bot token is not utf8")
    }
}

fn to_custom_bot(row: CustomBotRow) -> CustomBot {
    CustomBot {
        id: row.id,
        owner_user_id: row.owner_user_id,
        label: row.label,
        bot_user_id: row.bot_user_id,
        bot_username: row.bot_username,
        application_id: row.application_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn normalize_key(value: &str) -> Result<[u8; 32]> {
    if value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()) {
        let mut output = [0u8; 32];
        hex::decode_to_slice(value, &mut output).context("decode secrets master key")?;
        return Ok(output);
    }
    let bytes = value.as_bytes();
    if bytes.len() != 32 {
        return Err(eyre!("secrets master key must be 32 bytes or 64-char hex"));
    }
    let mut output = [0u8; 32];
    output.copy_from_slice(bytes);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{
        CustomBotService, custom_bot_deployment_id, custom_bot_id_from_deployment_id, normalize_key,
    };
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[test]
    fn custom_bot_scope_roundtrips() {
        let id = Uuid::new_v4();
        let scope = custom_bot_deployment_id(id);
        assert_eq!(custom_bot_id_from_deployment_id(&scope), Some(id));
    }

    #[tokio::test]
    async fn token_encryption_roundtrips() {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://flora:flora@localhost/flora")
            .expect("lazy pool");
        let service = CustomBotService {
            db,
            key: normalize_key("01234567890123456789012345678901").expect("key"),
        };
        let (ciphertext, nonce) = service.encrypt("secret token").expect("encrypt");
        assert_ne!(ciphertext, b"secret token");
        assert_eq!(
            service.decrypt(&ciphertext, &nonce).expect("decrypt"),
            "secret token"
        );
    }
}
