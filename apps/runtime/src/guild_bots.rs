use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, Result, eyre};
use rand::RngCore;
use serde::Serialize;
use serenity::{all::Token, http::Http, model::id::GuildId};
use sqlx::{FromRow, Pool, Postgres};

#[derive(Debug, Clone, Serialize)]
pub struct GuildBotBinding {
    pub guild_id: String,
    pub owner_user_id: String,
    pub bot_user_id: String,
    pub bot_username: String,
    pub application_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GuildBotBindingWithToken {
    pub binding: GuildBotBinding,
    pub bot_token: String,
}

#[derive(FromRow)]
struct GuildBotBindingRow {
    guild_id: String,
    owner_user_id: String,
    bot_user_id: String,
    bot_username: String,
    application_id: String,
    token_ciphertext: Vec<u8>,
    token_nonce: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct GuildBotService {
    db_pool: Pool<Postgres>,
    key_bytes: [u8; 32],
}

impl GuildBotService {
    pub fn new(db_pool: Pool<Postgres>, master_key: String) -> Result<Self> {
        let key_bytes = normalize_key(master_key)?;
        Ok(Self { db_pool, key_bytes })
    }

    pub async fn upsert_binding(
        &self,
        guild_id: &str,
        owner_user_id: &str,
        bot_token: &str,
    ) -> Result<GuildBotBinding> {
        let (bot_user_id, bot_username, application_id) =
            validate_bot_token_for_guild(guild_id, bot_token).await?;
        let (token_ciphertext, token_nonce) = self.encrypt_token(bot_token)?;

        let row = sqlx::query_as::<_, GuildBotBindingRow>(
            r#"
            INSERT INTO guild_bot_bindings (
                guild_id,
                owner_user_id,
                bot_user_id,
                bot_username,
                application_id,
                token_ciphertext,
                token_nonce
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (guild_id) DO UPDATE
            SET owner_user_id = EXCLUDED.owner_user_id,
                bot_user_id = EXCLUDED.bot_user_id,
                bot_username = EXCLUDED.bot_username,
                application_id = EXCLUDED.application_id,
                token_ciphertext = EXCLUDED.token_ciphertext,
                token_nonce = EXCLUDED.token_nonce,
                updated_at = NOW()
            RETURNING
                guild_id,
                owner_user_id,
                bot_user_id,
                bot_username,
                application_id,
                token_ciphertext,
                token_nonce,
                created_at,
                updated_at
            "#,
        )
        .bind(guild_id)
        .bind(owner_user_id)
        .bind(bot_user_id)
        .bind(bot_username)
        .bind(application_id)
        .bind(token_ciphertext)
        .bind(token_nonce)
        .fetch_one(&self.db_pool)
        .await
        .context("store guild bot binding")?;

        Ok(to_binding(row))
    }

    pub async fn get_binding(&self, guild_id: &str) -> Result<Option<GuildBotBinding>> {
        let row = sqlx::query_as::<_, GuildBotBindingRow>(
            r#"
            SELECT
                guild_id,
                owner_user_id,
                bot_user_id,
                bot_username,
                application_id,
                token_ciphertext,
                token_nonce,
                created_at,
                updated_at
            FROM guild_bot_bindings
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db_pool)
        .await
        .context("load guild bot binding")?;

        Ok(row.map(to_binding))
    }

    pub async fn get_binding_with_token(
        &self,
        guild_id: &str,
    ) -> Result<Option<GuildBotBindingWithToken>> {
        let row = sqlx::query_as::<_, GuildBotBindingRow>(
            r#"
            SELECT
                guild_id,
                owner_user_id,
                bot_user_id,
                bot_username,
                application_id,
                token_ciphertext,
                token_nonce,
                created_at,
                updated_at
            FROM guild_bot_bindings
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db_pool)
        .await
        .context("load guild bot binding with token")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let bot_token = self
            .decrypt_token(&row.token_ciphertext, &row.token_nonce)
            .context("decrypt guild bot token")?;

        Ok(Some(GuildBotBindingWithToken {
            binding: to_binding(row),
            bot_token,
        }))
    }

    pub async fn list_bindings_with_tokens(&self) -> Result<Vec<GuildBotBindingWithToken>> {
        let rows = sqlx::query_as::<_, GuildBotBindingRow>(
            r#"
            SELECT
                guild_id,
                owner_user_id,
                bot_user_id,
                bot_username,
                application_id,
                token_ciphertext,
                token_nonce,
                created_at,
                updated_at
            FROM guild_bot_bindings
            ORDER BY guild_id ASC
            "#,
        )
        .fetch_all(&self.db_pool)
        .await
        .context("list guild bot bindings")?;

        let mut bindings = Vec::with_capacity(rows.len());
        for row in rows {
            let bot_token = self
                .decrypt_token(&row.token_ciphertext, &row.token_nonce)
                .context("decrypt guild bot token")?;
            bindings.push(GuildBotBindingWithToken {
                binding: to_binding(row),
                bot_token,
            });
        }
        Ok(bindings)
    }

    pub async fn delete_binding(&self, guild_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM guild_bot_bindings WHERE guild_id = $1")
            .bind(guild_id)
            .execute(&self.db_pool)
            .await
            .context("delete guild bot binding")?;

        Ok(result.rows_affected() > 0)
    }

    fn encrypt_token(&self, token: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        let cipher = XChaCha20Poly1305::new(&self.key_bytes.into());

        let mut nonce_bytes = [0u8; 24];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, token.as_bytes())
            .context("encrypt token")?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    fn decrypt_token(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String> {
        let nonce_bytes: [u8; 24] = nonce
            .try_into()
            .map_err(|_| eyre!("invalid token nonce length"))?;
        let cipher = XChaCha20Poly1305::new(&self.key_bytes.into());
        let plaintext = cipher
            .decrypt(XNonce::from_slice(&nonce_bytes), ciphertext)
            .context("decrypt token")?;
        String::from_utf8(plaintext).context("token is not utf8")
    }
}

async fn validate_bot_token_for_guild(
    guild_id: &str,
    token_raw: &str,
) -> Result<(String, String, String)> {
    let token: Token = token_raw
        .parse()
        .map_err(|err: serenity::secrets::TokenError| eyre!(err))
        .context("invalid bot token")?;

    let http = Http::new(token);
    let bot_user = http
        .get_current_user()
        .await
        .context("failed to fetch bot user")?;
    let app = http
        .get_current_application_info()
        .await
        .context("failed to fetch bot application")?;

    let guild_id_num: u64 = guild_id.parse().map_err(|_| eyre!("invalid guild id"))?;
    let guild_id = GuildId::new(guild_id_num);

    http.get_guild(guild_id)
        .await
        .context("bot is not present in guild")?;

    Ok((
        bot_user.id.get().to_string(),
        bot_user.name.to_string(),
        app.id.get().to_string(),
    ))
}

fn to_binding(row: GuildBotBindingRow) -> GuildBotBinding {
    GuildBotBinding {
        guild_id: row.guild_id,
        owner_user_id: row.owner_user_id,
        bot_user_id: row.bot_user_id,
        bot_username: row.bot_username,
        application_id: row.application_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn normalize_key(master_key: String) -> Result<[u8; 32]> {
    if master_key.len() == 64 && master_key.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut out = [0u8; 32];
        hex::decode_to_slice(master_key, &mut out as &mut [u8]).context("decode hex master key")?;
        return Ok(out);
    }

    let raw = master_key.as_bytes();
    if raw.len() != 32 {
        return Err(eyre!(
            "SECRETS_MASTER_KEY must be 32 bytes or 64-char hex (got {})",
            raw.len()
        ));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(raw);
    Ok(out)
}
