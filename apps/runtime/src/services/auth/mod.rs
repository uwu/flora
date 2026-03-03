use axum::http::Uri;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use color_eyre::eyre::{Context, Result, eyre};
use cookie::{Cookie, SameSite};
use fred::{
    prelude::*,
    types::{ConnectHandle, Expiration},
};
use hmac::{Hmac, Mac};
use rand::{Rng, distributions::Alphanumeric};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use time::Duration as TimeDuration;
use tracing::warn;

type HmacSha256 = Hmac<Sha256>;

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
#[derive(Debug, Deserialize, Clone)]
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

/// Service responsible for OAuth exchanges and session management.
#[derive(Clone)]
pub struct AuthService {
    config: AuthConfig,
    cache_pool: fred::prelude::Pool,
    _cache_task: Arc<ConnectHandle>,
    http: HttpClient,
    session_key: Vec<u8>,
}

impl AuthService {
    pub fn new(
        config: AuthConfig,
        cache_pool: fred::prelude::Pool,
        cache_task: ConnectHandle,
    ) -> Result<Self> {
        if config.session_secret.len() < 32 {
            return Err(eyre!("SESSION_SECRET must be at least 32 characters long"));
        }

        let session_key = config.session_secret.as_bytes().to_vec();
        let http = HttpClient::builder()
            .user_agent("flora-oauth/0.1")
            .build()
            .wrap_err("failed to build http client")?;

        Ok(Self {
            cache_pool,
            _cache_task: Arc::new(cache_task),
            http,
            session_key,
            config,
        })
    }

    pub fn authorization_url(&self, state: &str) -> Result<Uri> {
        let scopes = "identify guilds guilds.members.read";
        let encoded_redirect = urlencoding::encode(&self.config.redirect_uri);
        let encoded_scopes = urlencoding::encode(scopes);
        let url = format!(
            "https://discord.com/api/oauth2/authorize?client_id={}&response_type=code&redirect_uri={encoded_redirect}&scope={encoded_scopes}&state={state}&prompt=consent",
            self.config.client_id
        );
        url.parse()
            .map_err(|err| eyre!("invalid authorization url: {err}"))
    }

    pub fn generate_state(&self) -> String {
        self.random_token(24)
    }

    pub async fn issue_state(&self, state: String) -> Result<()> {
        let key = format!("oauth:state:{state}");
        self.cache_pool
            .set::<(), _, _>(key, "1", Some(Expiration::EX(600)), None, false)
            .await
            .wrap_err("failed to cache oauth state")?;
        Ok(())
    }

    pub async fn consume_state(&self, state: &str) -> Result<bool> {
        let key = format!("oauth:state:{state}");
        let removed: i64 = self
            .cache_pool
            .del::<i64, _>(key.clone())
            .await
            .wrap_err("failed to consume oauth state")?;
        Ok(removed > 0)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
        ];

        let res = self
            .http
            .post("https://discord.com/api/oauth2/token")
            .form(&params)
            .send()
            .await
            .wrap_err("failed to request discord token")?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("discord token exchange failed: {}", body));
        }

        res.json::<TokenResponse>()
            .await
            .wrap_err("failed to decode token exchange response")
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let res = self
            .http
            .post("https://discord.com/api/oauth2/token")
            .form(&params)
            .send()
            .await
            .wrap_err("failed to refresh discord token")?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("discord token refresh failed: {}", body));
        }

        res.json::<TokenResponse>()
            .await
            .wrap_err("failed to decode refresh response")
    }

    pub async fn fetch_user(&self, access_token: &str) -> Result<DiscordUser> {
        let res = self
            .http
            .get("https://discord.com/api/users/@me")
            .bearer_auth(access_token)
            .send()
            .await
            .wrap_err("failed to request discord user")?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("discord identify failed: {}", body));
        }

        res.json::<DiscordUser>()
            .await
            .wrap_err("failed to decode discord user")
    }

    pub async fn fetch_user_guilds(&self, access_token: &str) -> Result<Vec<UserGuild>> {
        let res = self
            .http
            .get("https://discord.com/api/users/@me/guilds")
            .bearer_auth(access_token)
            .send()
            .await
            .wrap_err("failed to request user guilds")?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("failed to list guilds: {}", body));
        }

        let body = res.text().await.wrap_err("failed to read response")?;
        serde_json::from_str::<Vec<UserGuild>>(&body).wrap_err_with(|| {
            let preview = if body.len() > 500 {
                &body[..500]
            } else {
                &body
            };
            format!("failed to decode user guilds. First 500 chars: {}", preview)
        })
    }

    pub async fn fetch_guild_member(
        &self,
        guild_id: &str,
        access_token: &str,
    ) -> Result<Option<CurrentUserGuildMember>> {
        let url = format!("https://discord.com/api/users/@me/guilds/{guild_id}/member");
        let res = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .wrap_err("failed to request guild membership")?;

        match res.status().as_u16() {
            200 => {
                let body = res
                    .text()
                    .await
                    .wrap_err("failed to read guild member response")?;
                serde_json::from_str::<CurrentUserGuildMember>(&body)
                    .map(Some)
                    .wrap_err_with(|| {
                        let preview = if body.len() > 1000 {
                            &body[..1000]
                        } else {
                            &body
                        };
                        format!(
                            "failed to decode guild membership. Full response: {}",
                            preview
                        )
                    })
            }
            401 => Ok(None),
            403 | 404 => Ok(None),
            other => {
                let body = res.text().await.unwrap_or_default();
                warn!(target: "flora:auth", guild_id, status = other, body, "unexpected guild member response");
                Ok(None)
            }
        }
    }

    pub async fn store_session(&self, mut session: Session) -> Result<String> {
        let session_id = self.random_token(24);
        let token = self.sign_session_token(&session_id)?;

        // Align session TTL with OAuth expiry but cap to configured TTL.
        let now = Utc::now();
        let access_ttl = (session.expires_at - now).num_seconds().max(60) as u64;
        let ttl = access_ttl.min(self.config.session_ttl_secs);
        session.expires_at = now + Duration::seconds(ttl as i64);

        let key = session_cache_key(&session_id);
        let value = serde_json::to_string(&session)?;
        let ttl_i64 = ttl as i64;
        self.cache_pool
            .set::<(), _, _>(key, value, Some(Expiration::EX(ttl_i64)), None, false)
            .await
            .wrap_err("failed to persist session")?;

        Ok(token)
    }

    pub async fn get_session(&self, token: &str) -> Result<Option<Session>> {
        let session_id = match self.verify_session_token(token) {
            Ok(id) => id,
            Err(err) => {
                warn!(target: "flora:auth", ?err, "invalid session signature");
                return Ok(None);
            }
        };

        let key = session_cache_key(&session_id);
        let value: Option<String> = self
            .cache_pool
            .get(key.clone())
            .await
            .wrap_err("failed to load session")?;

        let Some(raw) = value else {
            return Ok(None);
        };

        let mut session: Session =
            serde_json::from_str(&raw).wrap_err("failed to decode session json")?;

        // Refresh token if expired and refresh is available.
        if session.expires_at <= Utc::now() {
            if let Some(refresh) = session.refresh_token.clone() {
                match self.refresh_token(&refresh).await {
                    Ok(tokens) => {
                        session.access_token = tokens.access_token;
                        session.refresh_token = tokens.refresh_token.or(session.refresh_token);
                        session.expires_at = Utc::now() + Duration::seconds(tokens.expires_in - 30);
                        session.scope = tokens.scope;
                        session.token_type = tokens.token_type;

                        let value = serde_json::to_string(&session)?;
                        let ttl = self.config.session_ttl_secs as i64;
                        self.cache_pool
                            .set::<(), _, _>(
                                key.clone(),
                                value,
                                Some(Expiration::EX(ttl)),
                                None,
                                false,
                            )
                            .await?;
                    }
                    Err(err) => {
                        warn!(target: "flora:auth", ?err, "failed to refresh expired session");
                        self.cache_pool.del::<(), _>(key).await.ok();
                        return Ok(None);
                    }
                }
            } else {
                self.cache_pool.del::<(), _>(key).await.ok();
                return Ok(None);
            }
        } else {
            // Touch TTL to keep active sessions warm.
            let ttl = self.config.session_ttl_secs as i64;
            self.cache_pool.expire::<(), _>(key, ttl, None).await.ok();
        }

        Ok(Some(session))
    }

    pub fn build_session_cookie(&self, token: &str) -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE, token.to_string()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(self.config.cookie_secure)
            .path("/")
            .max_age(TimeDuration::seconds(self.config.session_ttl_secs as i64))
            .build()
    }

    pub fn build_state_cookie(&self, state: &str) -> Cookie<'static> {
        Cookie::build((STATE_COOKIE, state.to_string()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(self.config.cookie_secure)
            .path("/")
            .max_age(TimeDuration::minutes(10))
            .build()
    }

    fn random_token(&self, len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    fn sign_session_token(&self, session_id: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(&self.session_key)
            .map_err(|err| eyre!("invalid session key: {err}"))?;
        mac.update(session_id.as_bytes());
        let signature = mac.finalize().into_bytes();
        let sig = URL_SAFE_NO_PAD.encode(signature);
        Ok(format!("{session_id}.{sig}"))
    }

    fn verify_session_token(&self, token: &str) -> Result<String> {
        let mut parts = token.split('.');
        let Some(session_id) = parts.next() else {
            return Err(eyre!("missing session id"));
        };
        let Some(sig) = parts.next() else {
            return Err(eyre!("missing signature"));
        };

        let provided = URL_SAFE_NO_PAD
            .decode(sig.as_bytes())
            .map_err(|err| eyre!("invalid signature encoding: {err}"))?;

        let mut mac = HmacSha256::new_from_slice(&self.session_key)
            .map_err(|err| eyre!("invalid session key: {err}"))?;
        mac.update(session_id.as_bytes());
        mac.verify_slice(&provided)
            .map_err(|_| eyre!("signature mismatch"))?;

        Ok(session_id.to_string())
    }
}

fn session_cache_key(id: &str) -> String {
    format!("session:{id}")
}
