use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
};
use chrono::Utc;
use cookie::Cookie;
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::{
    handlers::{
        error::ApiError,
        response::{ApiJson, ApiRedirectWithCookies},
    },
    services::auth::{DiscordUser, SESSION_COOKIE, Session},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index_handler))
        .route("/log-me-in/{username}", get(log_me_in_handler))
        .route("/log-me-out", get(log_me_out_handler))
        .route("/preflight", get(preflight_handler))
}

#[derive(Debug, Serialize)]
struct DevIndex {
    endpoints: Vec<&'static str>,
}

async fn index_handler() -> ApiJson<DevIndex> {
    ApiJson(Json(DevIndex {
        endpoints: vec![
            "/__dev/log-me-in/<username>?returnTo=/",
            "/__dev/log-me-out?returnTo=/",
            "/__dev/preflight",
        ],
    }))
}

#[derive(Debug, Deserialize)]
struct ReturnToQuery {
    #[serde(rename = "returnTo")]
    return_to: Option<String>,
}

async fn log_me_in_handler(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<ReturnToQuery>,
) -> Result<ApiRedirectWithCookies, ApiError> {
    let display_name = username.trim().trim_end_matches("@flora.local");
    let username = if display_name.is_empty() {
        "agent"
    } else {
        display_name
    };
    let session = Session {
        user: DiscordUser {
            id: "100000000000000001".to_string(),
            username: username.to_string(),
            global_name: Some(username.to_string()),
            avatar: None,
        },
        access_token: "dev-access-token".to_string(),
        refresh_token: None,
        token_type: "Bearer".to_string(),
        scope: "identify guilds guilds.members.read".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
    };
    let session_token = state
        .auth
        .store_session(session)
        .await
        .map_err(ApiError::internal)?;

    Ok(ApiRedirectWithCookies {
        response: Redirect::to(safe_return_to(query.return_to).as_str()).into_response(),
        cookies: vec![state.auth.build_session_cookie(&session_token)],
    })
}

async fn log_me_out_handler(Query(query): Query<ReturnToQuery>) -> ApiRedirectWithCookies {
    let removal = Cookie::build(SESSION_COOKIE)
        .path("/")
        .max_age(Duration::seconds(0))
        .build();

    ApiRedirectWithCookies {
        response: Redirect::to(safe_return_to(query.return_to).as_str()).into_response(),
        cookies: vec![removal],
    }
}

#[derive(Debug, Serialize)]
struct DevPreflight {
    debug: bool,
    auth_shortcuts: Vec<&'static str>,
}

async fn preflight_handler() -> ApiJson<DevPreflight> {
    ApiJson(Json(DevPreflight {
        debug: true,
        auth_shortcuts: vec![
            "/__dev/log-me-in/agent?returnTo=/",
            "/__dev/log-me-out?returnTo=/",
        ],
    }))
}

fn safe_return_to(value: Option<String>) -> String {
    let Some(value) = value else {
        return "/".to_string();
    };

    if value.starts_with('/') && !value.starts_with("//") {
        value
    } else {
        "/".to_string()
    }
}
