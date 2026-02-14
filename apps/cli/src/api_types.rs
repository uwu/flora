use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct DeploymentFile {
    pub(crate) path: String,
    pub(crate) contents: String,
}

#[derive(Serialize)]
pub(crate) struct DeploymentRequest {
    pub(crate) entry: String,
    pub(crate) files: Vec<DeploymentFile>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct DeploymentResponse {
    pub(crate) guild_id: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) entry: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct HealthResponse(pub(crate) String);

#[derive(Serialize)]
pub(crate) struct CreateStoreRequest {
    pub(crate) guild_id: String,
    pub(crate) store_name: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateStoreResponse {
    pub(crate) store: KvStore,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct KvStore {
    pub(crate) id: String,
    pub(crate) guild_id: String,
    pub(crate) store_name: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Serialize)]
pub(crate) struct SetValueRequest {
    pub(crate) value: String,
    pub(crate) expiration: Option<i64>,
    pub(crate) metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub(crate) struct GetValueResponse {
    pub(crate) value: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ListKeysResponse {
    pub(crate) keys: Vec<KvKeyInfo>,
    pub(crate) list_complete: bool,
    pub(crate) cursor: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct KvKeyInfo {
    pub(crate) name: String,
    pub(crate) expiration: Option<i64>,
    pub(crate) metadata: Option<serde_json::Value>,
}

/// A log entry from the runtime.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LogEntry {
    /// Timestamp in milliseconds since Unix epoch.
    pub(crate) timestamp: i64,
    /// Log level (trace, debug, info, warn, error).
    pub(crate) level: String,
    /// Target/module that produced the log.
    pub(crate) target: String,
    /// Guild ID if applicable.
    pub(crate) guild_id: Option<String>,
    /// Log message.
    pub(crate) message: String,
}
