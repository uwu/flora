use crate::secrets::SecretsRuntimeData;
use deno_error::JsErrorBox;
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static CURRENT_SECRETS: RefCell<Option<Arc<SecretsRuntimeData>>> = RefCell::new(None);
}

/// Sets thread-local secrets for the duration of a dispatch.
pub(super) struct SecretScope;

impl SecretScope {
    pub(super) fn enter(data: Arc<SecretsRuntimeData>) -> Self {
        CURRENT_SECRETS.with(|cell| {
            *cell.borrow_mut() = Some(data);
        });
        Self
    }
}

impl Drop for SecretScope {
    fn drop(&mut self) {
        CURRENT_SECRETS.with(|cell| {
            cell.borrow_mut().take();
        });
    }
}

pub(super) fn secret_request_builder_hook(
    request: &mut http::Request<deno_fetch::ReqBody>,
) -> Result<(), JsErrorBox> {
    let secrets = CURRENT_SECRETS.with(|cell| cell.borrow().clone());
    let Some(secrets) = secrets else {
        return Ok(());
    };

    let mut matched_allowed: Vec<Vec<String>> = Vec::new();

    let uri_string = request.uri().to_string();
    let (new_uri, uri_matches) = substitute_placeholders(&uri_string, &secrets);
    matched_allowed.extend(uri_matches);
    if new_uri != uri_string {
        let parsed = new_uri
            .parse()
            .map_err(|_| JsErrorBox::generic("failed to parse uri after secret substitution"))?;
        *request.uri_mut() = parsed;
    }

    let header_keys: Vec<_> = request.headers().keys().cloned().collect();
    for name in header_keys {
        if let Some(value) = request.headers_mut().get_mut(&name) {
            let Ok(orig) = value.to_str() else {
                continue;
            };
            let (replaced, hits) = substitute_placeholders(orig, &secrets);
            matched_allowed.extend(hits);
            if replaced != orig {
                *value = http::HeaderValue::from_str(&replaced)
                    .map_err(|_| JsErrorBox::generic("invalid header after secret substitution"))?;
            }
        }
    }

    if !matched_allowed.is_empty() {
        let host = request.uri().host();
        for allow in matched_allowed {
            if !allow.is_empty() && !host_allowed(host, &allow) {
                return Err(JsErrorBox::generic("secret not allowed for request host"));
            }
        }
    }

    Ok(())
}

pub(super) fn substitute_placeholders(
    input: &str,
    secrets: &SecretsRuntimeData,
) -> (String, Vec<Vec<String>>) {
    let mut output = input.to_string();
    let mut matched = Vec::new();
    for (placeholder, entry) in secrets.by_placeholder.iter() {
        if output.contains(placeholder) {
            output = output.replace(placeholder, &entry.value);
            matched.push(entry.allowed_hosts.clone());
        }
    }
    (output, matched)
}

pub(super) fn host_allowed(host: Option<&str>, allowlist: &[String]) -> bool {
    if allowlist.is_empty() {
        return true;
    }
    let Some(host) = host else {
        return false;
    };
    allowlist.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix("*.") {
            host.ends_with(suffix)
        } else {
            host == pattern
        }
    })
}
