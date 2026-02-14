use crate::{
    api_types::{DeploymentRequest, DeploymentResponse},
    auth::AuthRequestBuilder,
    config::CliConfig,
    files::{collect_files, path_to_relative},
};
use color_eyre::eyre::{Result, eyre};
use reqwest::Client;
use std::path::{Path, PathBuf};

pub(crate) async fn deploy(
    client: &Client,
    config: &CliConfig,
    guild: String,
    entry: PathBuf,
    root: Option<PathBuf>,
) -> Result<()> {
    let entry = entry
        .canonicalize()
        .map_err(|err| eyre!("Failed to read entry {}: {err}", entry.display()))?;
    let root = root
        .map(|path| path.canonicalize())
        .transpose()
        .map_err(|err| eyre!("Failed to resolve root: {err}"))?
        .unwrap_or_else(|| {
            entry
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });

    let files = collect_files(&root)?;
    let entry_rel = path_to_relative(&entry, &root)?;
    let body = DeploymentRequest {
        entry: entry_rel,
        files,
    };

    let url = format!("{}/deployments/{guild}", config.api_url);

    let resp = client
        .post(url)
        .maybe_bearer(&config.token)?
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<DeploymentResponse>()
        .await?;

    println!(
        "Deployed guild {} entry={} updated={}",
        resp.guild_id, resp.entry, resp.updated_at
    );
    Ok(())
}

pub(crate) async fn get(client: &Client, config: &CliConfig, guild: String) -> Result<()> {
    let url = format!("{}/deployments/{guild}", config.api_url);
    let resp = client
        .get(url)
        .maybe_bearer(&config.token)?
        .send()
        .await?
        .error_for_status()?;
    let deployment = resp.json::<DeploymentResponse>().await?;
    println!(
        "Guild {}\n  entry: {}\n  created: {}\n  updated: {}",
        deployment.guild_id, deployment.entry, deployment.created_at, deployment.updated_at
    );
    Ok(())
}

pub(crate) async fn list(client: &Client, config: &CliConfig) -> Result<()> {
    let url = format!("{}/deployments", config.api_url);
    let deployments = client
        .get(url)
        .maybe_bearer(&config.token)?
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<DeploymentResponse>>()
        .await?;

    if deployments.is_empty() {
        println!("No deployments found");
    } else {
        for d in deployments {
            println!(
                "{} entry={} created={} updated={}",
                d.guild_id, d.entry, d.created_at, d.updated_at
            );
        }
    }
    Ok(())
}

pub(crate) async fn health(client: &Client, config: &CliConfig) -> Result<()> {
    let url = format!("{}/health", config.api_url);
    let resp = client
        .get(url)
        .maybe_bearer(&config.token)?
        .send()
        .await?
        .error_for_status()?;
    let body = resp.text().await?;
    println!("{body}");
    Ok(())
}
