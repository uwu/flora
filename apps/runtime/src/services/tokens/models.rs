use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
pub(crate) struct TokenRow {
    pub(crate) token_id: String,
    pub(crate) user_id: String,
    pub(crate) label: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) last_used_at: Option<DateTime<Utc>>,
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
