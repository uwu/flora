use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, Result, eyre};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serenity::{all::Token, http::Http};
use sqlx::{FromRow, Pool, Postgres};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerCustomBot {
    pub guild_id: String,
    pub configured_by_user_id: String,
    pub bot_user_id: String,
    pub bot_username: String,
    pub application_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ServerCustomBotWithToken {
    pub bot: ServerCustomBot,
    pub token: String,
}

#[derive(Debug, FromRow)]
struct ServerCustomBotRow {
    guild_id: String,
    configured_by_user_id: String,
    bot_user_id: String,
    bot_username: String,
    application_id: String,
    token_ciphertext: Vec<u8>,
    token_nonce: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ServerCustomBotService {
    db: Pool<Postgres>,
    key: [u8; 32],
}

impl ServerCustomBotService {
    pub fn new(db: Pool<Postgres>, master_key: String) -> Result<Self> {
        Ok(Self {
            db,
            key: normalize_key(&master_key)?,
        })
    }

    pub async fn validate_token(
        &self,
        guild_id: &str,
        raw_token: &str,
    ) -> Result<ValidatedServerBot> {
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
        let guild_id_num = guild_id.parse::<u64>().context("invalid guild id")?;
        http.get_guild(guild_id_num.into())
            .await
            .context("custom bot is not in this guild")?;
        Ok(ValidatedServerBot {
            token: token_text.to_string(),
            bot_user_id: user.id.get().to_string(),
            bot_username: user.name.to_string(),
            application_id: application.id.get().to_string(),
        })
    }

    pub async fn upsert(
        &self,
        guild_id: &str,
        configured_by_user_id: &str,
        validated: ValidatedServerBot,
    ) -> Result<ServerCustomBot> {
        let (ciphertext, nonce) = self.encrypt(&validated.token)?;
        let row = sqlx::query_as::<_, ServerCustomBotRow>(
            r#"
            INSERT INTO server_custom_bots (
                guild_id, configured_by_user_id, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (guild_id) DO UPDATE SET
                configured_by_user_id = EXCLUDED.configured_by_user_id,
                bot_user_id = EXCLUDED.bot_user_id,
                bot_username = EXCLUDED.bot_username,
                application_id = EXCLUDED.application_id,
                token_ciphertext = EXCLUDED.token_ciphertext,
                token_nonce = EXCLUDED.token_nonce,
                updated_at = NOW()
            RETURNING guild_id, configured_by_user_id, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
            "#,
        )
        .bind(guild_id)
        .bind(configured_by_user_id)
        .bind(validated.bot_user_id)
        .bind(validated.bot_username)
        .bind(validated.application_id)
        .bind(ciphertext)
        .bind(nonce)
        .fetch_one(&self.db)
        .await
        .context("store server custom bot")?;
        Ok(to_bot(row))
    }

    pub async fn get(&self, guild_id: &str) -> Result<Option<ServerCustomBot>> {
        Ok(self.get_row(guild_id).await?.map(to_bot))
    }

    pub async fn get_with_token(&self, guild_id: &str) -> Result<Option<ServerCustomBotWithToken>> {
        let Some(row) = self.get_row(guild_id).await? else {
            return Ok(None);
        };
        let token = self.decrypt(&row.token_ciphertext, &row.token_nonce)?;
        Ok(Some(ServerCustomBotWithToken {
            bot: to_bot(row),
            token,
        }))
    }

    pub async fn list_with_tokens(&self) -> Result<Vec<ServerCustomBotWithToken>> {
        let rows = sqlx::query_as::<_, ServerCustomBotRow>(
            r#"SELECT guild_id, configured_by_user_id, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
                FROM server_custom_bots ORDER BY created_at, guild_id"#,
        )
        .fetch_all(&self.db)
        .await?;
        rows.into_iter()
            .map(|row| {
                let token = self.decrypt(&row.token_ciphertext, &row.token_nonce)?;
                Ok(ServerCustomBotWithToken {
                    bot: to_bot(row),
                    token,
                })
            })
            .collect()
    }

    pub async fn delete(&self, guild_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM server_custom_bots WHERE guild_id = $1")
            .bind(guild_id)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn restore(&self, previous: Option<ServerCustomBotWithToken>) -> Result<()> {
        let Some(previous) = previous else {
            return Ok(());
        };
        let (ciphertext, nonce) = self.encrypt(&previous.token)?;
        sqlx::query(
            r#"UPDATE server_custom_bots SET
                configured_by_user_id = $2, bot_user_id = $3, bot_username = $4,
                application_id = $5, token_ciphertext = $6, token_nonce = $7,
                updated_at = $8 WHERE guild_id = $1"#,
        )
        .bind(&previous.bot.guild_id)
        .bind(&previous.bot.configured_by_user_id)
        .bind(&previous.bot.bot_user_id)
        .bind(&previous.bot.bot_username)
        .bind(&previous.bot.application_id)
        .bind(ciphertext)
        .bind(nonce)
        .bind(previous.bot.updated_at)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_row(&self, guild_id: &str) -> Result<Option<ServerCustomBotRow>> {
        Ok(sqlx::query_as::<_, ServerCustomBotRow>(
            r#"SELECT guild_id, configured_by_user_id, bot_user_id, bot_username, application_id,
                token_ciphertext, token_nonce, created_at, updated_at
                FROM server_custom_bots WHERE guild_id = $1"#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db)
        .await?)
    }

    fn encrypt(&self, value: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        let cipher = XChaCha20Poly1305::new(&self.key.into());
        let mut nonce = [0u8; 24];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ciphertext = cipher
            .encrypt(XNonce::from_slice(&nonce), value.as_bytes())
            .context("encrypt server custom bot token")?;
        Ok((ciphertext, nonce.to_vec()))
    }

    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String> {
        let nonce: [u8; 24] = nonce
            .try_into()
            .map_err(|_| eyre!("invalid bot token nonce"))?;
        let plaintext = XChaCha20Poly1305::new(&self.key.into())
            .decrypt(XNonce::from_slice(&nonce), ciphertext)
            .context("decrypt server custom bot token")?;
        String::from_utf8(plaintext).context("server custom bot token is not utf8")
    }
}

pub struct ValidatedServerBot {
    token: String,
    bot_user_id: String,
    bot_username: String,
    application_id: String,
}

fn to_bot(row: ServerCustomBotRow) -> ServerCustomBot {
    ServerCustomBot {
        guild_id: row.guild_id,
        configured_by_user_id: row.configured_by_user_id,
        bot_user_id: row.bot_user_id,
        bot_username: row.bot_username,
        application_id: row.application_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn normalize_key(value: &str) -> Result<[u8; 32]> {
    if value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()) {
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
    use super::{ServerCustomBotService, normalize_key};
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn token_encryption_roundtrips() {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://flora:flora@localhost/flora")
            .expect("lazy pool");
        let service = ServerCustomBotService {
            db,
            key: normalize_key("01234567890123456789012345678901").expect("key"),
        };
        let (ciphertext, nonce) = service.encrypt("server token").expect("encrypt");
        assert_ne!(ciphertext, b"server token");
        assert_eq!(
            service.decrypt(&ciphertext, &nonce).expect("decrypt"),
            "server token"
        );
    }
}
