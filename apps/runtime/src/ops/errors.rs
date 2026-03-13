use deno_error::{JsError, JsErrorBox};
use serenity::{Error as SerenityError, http::HttpError};

#[derive(Debug, thiserror::Error, JsError)]
#[class(generic)]
pub enum FloraError {
    #[property("code" = "DISCORD_RATE_LIMITED")]
    #[error("Discord rate limited for {route}")]
    RateLimited {
        retry_after_ms: u64,
        global: bool,
        route: String,
    },
    #[property("code" = "DISCORD_HTTP_ERROR")]
    #[error("Discord HTTP error ({status}): {message}")]
    DiscordHttp {
        status: u16,
        #[property = "discord_code"]
        code: i32,
        message: String,
    },
    #[property("code" = "SCOPE_FORBIDDEN")]
    #[error("Forbidden: {reason}")]
    ScopeForbidden { reason: String },
    #[property("code" = "INVALID_INPUT")]
    #[error("Invalid input for {field}: {reason}")]
    InvalidInput { field: String, reason: String },
}

impl FloraError {
    pub(crate) fn rate_limited(
        retry_after_ms: u64,
        global: bool,
        route: impl Into<String>,
    ) -> Self {
        Self::RateLimited {
            retry_after_ms,
            global,
            route: route.into(),
        }
    }

    pub(crate) fn discord_http(status: u16, code: i32, message: impl Into<String>) -> Self {
        Self::DiscordHttp {
            status,
            code,
            message: message.into(),
        }
    }

    pub(crate) fn scope_forbidden(reason: impl Into<String>) -> Self {
        Self::ScopeForbidden {
            reason: reason.into(),
        }
    }

    pub(crate) fn invalid_input(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn from_serenity_error(error: SerenityError, route: Option<&str>) -> Self {
        match error {
            SerenityError::Http(http_error) => match http_error {
                HttpError::UnsuccessfulRequest(response) => {
                    let status = response.status_code.as_u16();
                    let code = response.error.code.0 as i32;
                    let message = with_context(response.error.message.to_string(), route);
                    FloraError::discord_http(status, code, message)
                }
                other => FloraError::discord_http(500, 0, with_context(other.to_string(), route)),
            },
            other => FloraError::discord_http(500, 0, with_context(other.to_string(), route)),
        }
    }
}

impl From<FloraError> for JsErrorBox {
    fn from(err: FloraError) -> Self {
        JsErrorBox::from_err(err)
    }
}

fn with_context(message: String, route: Option<&str>) -> String {
    let Some(route) = route else {
        return message;
    };
    format!("{message} (route: {route})")
}
