use color_eyre::eyre::{Context, Result};
use eyre::eyre;
use reqwest::Client;

use super::models::{BuildResponse, CreateBuildResponse};

#[derive(Clone)]
pub struct BuildServiceClient {
    client: Client,
    base_url: String,
    secret: String,
}

impl BuildServiceClient {
    pub fn new(base_url: String, secret: String) -> Result<Self> {
        let client = Client::builder()
            .user_agent("flora-runtime/0.1")
            .build()
            .wrap_err("failed to build build-service http client")?;
        Ok(Self {
            client,
            base_url,
            secret,
        })
    }

    pub async fn create_build(
        &self,
        guild_id: &str,
        entry: &str,
        project_zip: Vec<u8>,
    ) -> Result<CreateBuildResponse> {
        let form = reqwest::multipart::Form::new()
            .text("guild_id", guild_id.to_string())
            .text("entry", entry.to_string())
            .part(
                "project_zip",
                reqwest::multipart::Part::bytes(project_zip).file_name("project.zip"),
            );

        let res = self
            .client
            .post(format!("{}/internal/builds", self.base_url))
            .bearer_auth(&self.secret)
            .multipart(form)
            .send()
            .await
            .wrap_err("failed to create build")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("build service error ({status}): {body}"));
        }

        res.json()
            .await
            .wrap_err("failed to decode create-build response")
    }

    pub async fn get_build(&self, build_id: &str) -> Result<Option<BuildResponse>> {
        let res = self
            .client
            .get(format!("{}/internal/builds/{build_id}", self.base_url))
            .bearer_auth(&self.secret)
            .send()
            .await
            .wrap_err("failed to get build")?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("build service error ({status}): {body}"));
        }

        res.json()
            .await
            .map(Some)
            .wrap_err("failed to decode build response")
    }

    pub async fn stream_logs(&self, build_id: &str) -> Result<reqwest::Response> {
        let res = self
            .client
            .get(format!("{}/internal/builds/{build_id}/logs", self.base_url))
            .bearer_auth(&self.secret)
            .send()
            .await
            .wrap_err("failed to stream build logs")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(eyre!("build service error ({status}): {body}"));
        }

        Ok(res)
    }
}
