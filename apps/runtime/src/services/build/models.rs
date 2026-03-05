use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBuildResponse {
    pub build_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub bundle: String,
    #[serde(rename = "sourceMap")]
    pub source_map: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildResponse {
    pub build_id: String,
    pub guild_id: String,
    pub entry: String,
    pub status: String,
    pub logs: Vec<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub artifact: Option<BuildArtifact>,
    pub error: Option<String>,
}
