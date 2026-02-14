use color_eyre::eyre::{Result, eyre};

pub(crate) trait AuthRequestBuilder {
    fn maybe_bearer(self, token: &Option<String>) -> Result<Self>
    where
        Self: Sized;
}

impl AuthRequestBuilder for reqwest::RequestBuilder {
    fn maybe_bearer(self, token: &Option<String>) -> Result<Self> {
        if let Some(t) = token {
            Ok(self.bearer_auth(t))
        } else {
            Err(eyre!(
                "Missing API token; run `flora login --token <token>`"
            ))
        }
    }
}
