use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cookie name used to persist the session token.
pub const SESSION_COOKIE: &str = "flora_session";
/// Cookie name used for OAuth state tracking.
pub const STATE_COOKIE: &str = "flora_oauth_state";

/// Discord OAuth client configuration.
#[derive(Clone)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub session_secret: String,
    pub session_ttl_secs: u64,
    pub cookie_secure: bool,
}

/// Persisted session data stored in Valkey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user: DiscordUser,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: String,
    pub expires_at: DateTime<Utc>,
}

/// Minimal Discord user profile shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub scope: String,
    pub token_type: String,
}

/// Membership information returned by Discord for the current user in a guild.
#[derive(Debug, Deserialize, Clone)]
pub struct CurrentUserGuildMember {
    #[serde(default, deserialize_with = "deserialize_permission_field")]
    pub permissions: Option<String>,
}

fn deserialize_permission_field<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    use serde_json::Value;

    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(value.and_then(|v| match v {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }))
}

/// Guild entry returned by /users/@me/guilds.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserGuild {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default, deserialize_with = "deserialize_permission_string_or_number")]
    pub permissions: Option<String>,
    #[serde(default)]
    pub permissions_new: Option<String>,
}

fn deserialize_permission_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    use serde_json::Value;

    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(value.and_then(|v| match v {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }))
}
