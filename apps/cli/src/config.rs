use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CliConfig {
    pub(crate) api_url: String,
    pub(crate) token: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000/api".to_string(),
            token: None,
        }
    }
}
