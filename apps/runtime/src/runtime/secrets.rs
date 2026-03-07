use crate::services::secrets::SecretsRuntimeData;
use deno_error::JsErrorBox;
use std::{cell::RefCell, sync::Arc};
use url::{Host, Url};

thread_local! {
    static CURRENT_SECRETS: RefCell<Option<Arc<SecretsRuntimeData>>> = const { RefCell::new(None) };
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

    let Some(host) = host.and_then(normalize_host) else {
        return false;
    };

    allowlist.iter().any(|pattern| {
        let Some(pattern) = parse_allowed_pattern(pattern) else {
            return false;
        };

        match pattern {
            AllowedHostPattern::Exact(allowed) => host == allowed,
            AllowedHostPattern::WildcardSuffix(suffix) => {
                host == suffix || host.ends_with(&format!(".{suffix}"))
            }
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AllowedHostPattern {
    Exact(String),
    WildcardSuffix(String),
}

fn parse_allowed_pattern(pattern: &str) -> Option<AllowedHostPattern> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return None;
    }

    if let Some(suffix) = pattern.strip_prefix("*.") {
        let suffix = parse_host_from_pattern(suffix)?;
        return Some(AllowedHostPattern::WildcardSuffix(suffix));
    }

    let exact = parse_host_from_pattern(pattern)?;
    Some(AllowedHostPattern::Exact(exact))
}

fn parse_host_from_pattern(pattern: &str) -> Option<String> {
    if let Some(host) = Url::parse(pattern)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
    {
        return normalize_host(&host);
    }

    if let Some(host) = normalize_host(pattern) {
        return Some(host);
    }

    let candidate = format!("https://{pattern}");
    Url::parse(&candidate)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .and_then(|host| normalize_host(&host))
}

fn normalize_host(host: &str) -> Option<String> {
    Host::parse(host.trim()).ok().map(|host| host.to_string())
}

#[cfg(test)]
mod tests {
    use super::host_allowed;

    #[test]
    fn allows_exact_hosts() {
        assert!(host_allowed(
            Some("api.example.com"),
            &["api.example.com".to_string()]
        ));
        assert!(!host_allowed(
            Some("api.example.com"),
            &["other.example.com".to_string()]
        ));
    }

    #[test]
    fn wildcard_matches_subdomain_or_root_only() {
        assert!(host_allowed(
            Some("api.example.com"),
            &["*.example.com".to_string()]
        ));
        assert!(host_allowed(
            Some("example.com"),
            &["*.example.com".to_string()]
        ));
        assert!(!host_allowed(
            Some("evil-example.com"),
            &["*.example.com".to_string()]
        ));
        assert!(!host_allowed(
            Some("notexample.com"),
            &["*.example.com".to_string()]
        ));
    }

    #[test]
    fn parses_urls_in_allowlist() {
        assert!(host_allowed(
            Some("api.example.com"),
            &["https://api.example.com/path".to_string()]
        ));
        assert!(!host_allowed(
            Some("other.com"),
            &["https://api.example.com/path".to_string()]
        ));
    }
}
