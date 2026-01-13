use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use cookie::Cookie;
use serde::{Deserialize, Serialize};
use time::Duration;
use tracing::error;
use utoipa::{OpenApi, ToSchema};

use crate::{
    auth::{AuthService, DiscordUser, SESSION_COOKIE, STATE_COOKIE, Session},
    handlers::{
        error::ApiError,
        response::{ApiJson, ApiRedirect, ApiRedirectWithCookies},
    },
    state::AppState,
};

/// Authentication routes.
#[derive(OpenApi)]
#[openapi(
    paths(login_handler, callback_handler, me_handler),
    components(schemas(AuthUser, AuthResponse, crate::handlers::error::ErrorResponse)),
    tags((name = "auth", description = "Discord authentication"))
)]
pub struct AuthApi;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/login", axum::routing::get(login_handler))
        .route("/callback", axum::routing::get(callback_handler))
        .route("/me", axum::routing::get(me_handler))
}

/// API representation of the authenticated user.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

impl From<DiscordUser> for AuthUser {
    fn from(value: DiscordUser) -> Self {
        Self {
            id: value.id,
            username: value.username,
            global_name: value.global_name,
            avatar: value.avatar,
        }
    }
}

/// Response envelope for /auth/me.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: AuthUser,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

/// Begin Discord OAuth flow and set a short-lived state cookie.
#[utoipa::path(
    get,
    path = "/login",
    tag = "auth",
    responses(
        (status = 302, description = "Redirect to Discord")
    )
)]
pub async fn login_handler(State(state): State<AppState>) -> Result<ApiRedirect, ApiError> {
    let oauth_state = state.auth.generate_state();
    if let Err(err) = state.auth.issue_state(oauth_state.clone()).await {
        error!(target: "flora:api", ?err, "failed to store oauth state");
        return Err(ApiError::internal(err));
    }

    let url = state
        .auth
        .authorization_url(&oauth_state)
        .map_err(ApiError::internal)?;
    let mut response = Redirect::to(url.to_string().as_str()).into_response();
    attach_cookie(&mut response, state.auth.build_state_cookie(&oauth_state));
    Ok(ApiRedirect { response })
}

/// Handle Discord OAuth callback, mint a session cookie, and redirect to the frontend dashboard.
#[utoipa::path(
    get,
    path = "/callback",
    tag = "auth",
    params(
        ("code" = String, Query, description = "Discord authorization code"),
        ("state" = String, Query, description = "Opaque state value returned by Discord")
    ),
    responses(
        (status = 302, description = "Login succeeded, redirecting to dashboard"),
        (status = 401, description = "Invalid or expired state", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn callback_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Result<ApiRedirectWithCookies, ApiError> {
    let Some(state_cookie) = cookie_value(&headers, STATE_COOKIE) else {
        return Err(ApiError::unauthorized("missing oauth state"));
    };
    if state_cookie != query.state {
        return Err(ApiError::unauthorized("oauth state mismatch"));
    }

    if !state
        .auth
        .consume_state(&query.state)
        .await
        .map_err(ApiError::internal)?
    {
        return Err(ApiError::unauthorized("expired oauth state"));
    }

    let tokens = state
        .auth
        .exchange_code(&query.code)
        .await
        .map_err(ApiError::internal)?;
    let user = state
        .auth
        .fetch_user(&tokens.access_token)
        .await
        .map_err(ApiError::internal)?;

    let session = Session {
        user: user.clone(),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        scope: tokens.scope,
        expires_at: Utc::now() + chrono::Duration::seconds(tokens.expires_in - 30),
    };

    let session_token = state
        .auth
        .store_session(session)
        .await
        .map_err(ApiError::internal)?;
    let mut removal = Cookie::build(STATE_COOKIE);
    removal = removal.path("/").max_age(Duration::seconds(0));

    let redirect_response = Redirect::to("/").into_response();
    Ok(ApiRedirectWithCookies {
        response: redirect_response,
        cookies: vec![
            state.auth.build_session_cookie(&session_token),
            removal.build(),
        ],
    })
}

/// Return the currently authenticated user.
#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    responses(
        (status = 200, description = "Session is valid", body = AuthResponse),
        (status = 401, description = "No active session", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn me_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<AuthResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    let user = match identity.session {
        Some(session) => session.user,
        None => DiscordUser {
            id: identity.user_id,
            username: "".to_string(),
            global_name: None,
            avatar: None,
        },
    };
    Ok(ApiJson(Json(AuthResponse { user: user.into() })))
}

/// Load and validate the session from cookies.
pub async fn require_session(auth: &AuthService, headers: &HeaderMap) -> Result<Session, ApiError> {
    let Some(cookie) = cookie_value(headers, SESSION_COOKIE) else {
        return Err(ApiError::unauthorized("login required"));
    };

    let token = cookie.as_str();
    let session = auth.get_session(token).await.map_err(ApiError::internal)?;

    session.ok_or_else(|| ApiError::unauthorized("session expired"))
}

pub struct IdentityContext {
    pub user_id: String,
    pub access_token: Option<String>,
    pub session: Option<Session>,
}

/// Resolve caller identity from either a bearer token or a session cookie.
pub async fn require_identity(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<IdentityContext, ApiError> {
    if let Some(bearer) = bearer_token(headers)
        && let Some(token) = state
            .tokens
            .validate_token(bearer)
            .await
            .map_err(ApiError::internal)?
    {
        return Ok(IdentityContext {
            user_id: token.user_id,
            access_token: None,
            session: None,
        });
    }

    let session = require_session(&state.auth, headers).await?;
    Ok(IdentityContext {
        user_id: session.user.id.clone(),
        access_token: Some(session.access_token.clone()),
        session: Some(session),
    })
}

/// Ensure the user is an admin or has manage-guild permissions in the target guild.
pub async fn ensure_guild_admin(
    state: &AppState,
    identity: &IdentityContext,
    guild_id: &str,
) -> Result<(), ApiError> {
    // Prefer user OAuth token if available; otherwise fall back to bot http.
    let member = if let Some(access_token) = &identity.access_token {
        state
            .auth
            .fetch_guild_member(guild_id, access_token)
            .await
            .map_err(ApiError::internal)?
    } else {
        // bot lookup
        let guild_id_num: u64 = guild_id
            .parse()
            .map_err(|_| ApiError::bad_request("invalid guild id"))?;
        let user_id_num: u64 = identity
            .user_id
            .parse()
            .map_err(|_| ApiError::bad_request("invalid user id"))?;
        let member = state
            .http
            .get_member(guild_id_num.into(), user_id_num.into())
            .await
            .map_err(|err| ApiError::forbidden(format!("member fetch failed: {err}")))?;

        let permissions = if let Some(perms) = member.permissions {
            perms.bits()
        } else {
            let guild = state
                .http
                .get_guild(guild_id_num.into())
                .await
                .map_err(|err| ApiError::forbidden(format!("guild fetch failed: {err}")))?;
            guild.member_permissions(&member).bits()
        };

        Some(crate::auth::CurrentUserGuildMember {
            permissions: Some(permissions.to_string()),
        })
    };

    let Some(member) = member else {
        return Err(ApiError::forbidden("bot not in guild or user not a member"));
    };

    let perms = member
        .permissions
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or_default();
    if has_admin_permissions(perms) {
        Ok(())
    } else {
        Err(ApiError::forbidden("admin permission required"))
    }
}

/// True when the permission bitset contains administrator-level access.
pub fn has_admin_permissions(perms: u64) -> bool {
    const ADMINISTRATOR: u64 = 0x0000_0008;
    const MANAGE_GUILD: u64 = 0x0000_0020;
    perms & (ADMINISTRATOR | MANAGE_GUILD) != 0
}

fn attach_cookie(response: &mut Response, cookie: Cookie<'_>) {
    if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
        response.headers_mut().append(SET_COOKIE, value);
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookie::Cookie::split_parse(raw)
        .filter_map(Result::ok)
        .find(|c| c.name() == name)
        .map(|c| c.value().to_string())
}

#[cfg(test)]
mod tests {
    use super::has_admin_permissions;

    #[test]
    fn detects_admin_flags() {
        assert!(has_admin_permissions(0x8));
        assert!(has_admin_permissions(0x20));
        assert!(has_admin_permissions(0x28));
        assert!(!has_admin_permissions(0x4));
        assert!(!has_admin_permissions(0));
    }
}
