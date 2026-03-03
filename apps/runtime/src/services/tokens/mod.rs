use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Pool, Postgres};

/// Persisted API token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub token_id: String,
    pub user_id: String,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
struct TokenRow {
    token_id: String,
    user_id: String,
    label: Option<String>,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct TokenService {
    db: Pool<Postgres>,
}

impl TokenService {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    /// Create a token for a user and return the plaintext value.
    pub async fn create_token(&self, user_id: &str, label: Option<String>) -> Result<String> {
        let token_id = Self::random_token(12);
        let secret = Self::random_token(32);
        let plaintext = format!("{token_id}.{secret}");
        let token_hash = Self::hash(&plaintext)?;

        sqlx::query(
            r#"
            INSERT INTO user_tokens (token_id, user_id, label, token_hash)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&token_id)
        .bind(user_id)
        .bind(label)
        .bind(&token_hash)
        .execute(&self.db)
        .await?;

        Ok(plaintext)
    }

    pub async fn list_tokens(&self, user_id: &str) -> Result<Vec<UserToken>> {
        let rows = sqlx::query_as::<_, TokenRow>(
            r#"
            SELECT token_id, user_id, label, created_at, last_used_at
            FROM user_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(UserToken::from).collect())
    }

    pub async fn delete_token(&self, user_id: &str, token_id: &str) -> Result<bool> {
        let deleted = sqlx::query("DELETE FROM user_tokens WHERE user_id = $1 AND token_id = $2")
            .bind(user_id)
            .bind(token_id)
            .execute(&self.db)
            .await?;
        Ok(deleted.rows_affected() > 0)
    }

    /// Validate a bearer token string and return the associated user id.
    pub async fn validate_token(&self, bearer: &str) -> Result<Option<UserToken>> {
        if bearer.len() < 20 {
            return Ok(None);
        }
        let token_hash = Self::hash(bearer)?;

        let row = sqlx::query_as::<_, TokenRow>(
            r#"
            UPDATE user_tokens
            SET last_used_at = NOW()
            WHERE token_hash = $1
            RETURNING token_id, user_id, label, created_at, last_used_at, token_hash
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(UserToken::from))
    }

    fn random_token(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    fn hash(token: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let digest = hasher.finalize();
        Ok(hex::encode(digest))
    }
}

impl From<TokenRow> for UserToken {
    fn from(value: TokenRow) -> Self {
        Self {
            token_id: value.token_id,
            user_id: value.user_id,
            label: value.label,
            created_at: value.created_at,
            last_used_at: value.last_used_at,
        }
    }
}
