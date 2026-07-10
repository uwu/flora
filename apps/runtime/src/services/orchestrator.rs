use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Reserved deployment key used to persist the singleton orchestrator alongside deployments.
pub const ORCHESTRATOR_DEPLOYMENT_ID: &str = "__flora_orchestrator__";

/// A feature authorization question evaluated by the trusted orchestrator script.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureAuthorizationRequest {
    /// Stable feature identifier, such as `custom_bots`.
    pub feature: String,
    /// Discord user ID requesting access to the feature.
    pub user_id: String,
    /// Optional feature-specific context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}
