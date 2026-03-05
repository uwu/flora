use chrono::{DateTime, Utc};
use flora_macros::expose_payload;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use t0x::T0x;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KvStore {
    pub id: String,
    pub guild_id: String,
    pub store_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub(crate) struct KvStoreRow {
    pub(crate) id: sqlx::types::Uuid,
    pub(crate) guild_id: String,
    pub(crate) store_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

/// Metadata associated with a KV key.
#[expose_payload]
#[derive(Clone, Deserialize, ToSchema)]
pub struct RawKvKeyMetadata {
    /// Unix timestamp (seconds) when this key expires.
    pub expiration: Option<i64>,
    /// Arbitrary JSON metadata attached to the key.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Information about a single KV key.
#[expose_payload]
#[derive(Clone, Deserialize, ToSchema)]
pub struct RawKvKeyInfo {
    /// The key's name.
    pub name: String,
    /// Unix timestamp (seconds) when this key expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<i64>,
    /// Arbitrary JSON metadata attached to the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Result of listing keys in a KV store.
#[expose_payload]
#[derive(Clone, Deserialize, ToSchema)]
pub struct RawKvListKeysResult {
    /// The keys returned by this list operation.
    pub keys: Vec<RawKvKeyInfo>,
    /// Whether all matching keys have been returned.
    pub list_complete: bool,
    /// Cursor for fetching the next page of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
