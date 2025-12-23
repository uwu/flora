use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, eyre};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CliConfig {
    api_url: String,
    token: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self { api_url: "http://localhost:3000/api".to_string(), token: None }
    }
}

#[derive(Parser, Debug)]
#[command(name = "flora", about = "Deployment CLI for flora guild scripts")]
struct Cli {
    /// API base URL (env: FLORA_API_URL). Overrides config.
    #[arg(long, env = "FLORA_API_URL")]
    api_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deploy or update a guild script
    Deploy {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Path to the entry file (e.g. src/main.ts)
        entry: PathBuf,
        /// Optional root directory to package (defaults to entry's parent)
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Fetch a guild deployment
    Get {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
    },
    /// List all deployments
    List,
    /// Health check
    Health,
    /// Persist an API token for future commands
    Login {
        /// API token from /tokens
        token: String,
    },
}

#[derive(Serialize)]
struct DeploymentFile {
    path: String,
    contents: String,
}

#[derive(Serialize)]
struct DeploymentRequest {
    entry: String,
    files: Vec<DeploymentFile>,
}

#[derive(Deserialize, Debug)]
struct DeploymentResponse {
    guild_id: String,
    created_at: String,
    updated_at: String,
    entry: String,
}

#[derive(Deserialize, Debug)]
struct HealthResponse(String);

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let mut config: CliConfig = confy::load("flora", "cli")?;
    if let Some(url) = cli.api_url.clone() {
        config.api_url = url;
    }
    let client = Client::new();

    match cli.command {
        Commands::Deploy { guild, entry, root } => {
            deploy(&client, &config, guild, entry, root).await?
        }
        Commands::Get { guild } => get(&client, &config, guild).await?,
        Commands::List => list(&client, &config).await?,
        Commands::Health => health(&client, &config).await?,
        Commands::Login { token } => {
            config.token = Some(token);
            confy::store("flora", "cli", config)?;
            println!("Saved token to config");
        }
    }

    Ok(())
}

async fn deploy(
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
            entry.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf()
        });

    let files = collect_files(&root)?;
    let entry_rel = path_to_relative(&entry, &root)?;
    let body = DeploymentRequest { entry: entry_rel, files };

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

    println!("Deployed guild {} entry={} updated={}", resp.guild_id, resp.entry, resp.updated_at);
    Ok(())
}

async fn get(client: &Client, config: &CliConfig, guild: String) -> Result<()> {
    let url = format!("{}/deployments/{guild}", config.api_url);
    let resp = client.get(url).maybe_bearer(&config.token)?.send().await?.error_for_status()?;
    let deployment = resp.json::<DeploymentResponse>().await?;
    println!(
        "Guild {}\n  entry: {}\n  created: {}\n  updated: {}",
        deployment.guild_id, deployment.entry, deployment.created_at, deployment.updated_at
    );
    Ok(())
}

async fn list(client: &Client, config: &CliConfig) -> Result<()> {
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

async fn health(client: &Client, config: &CliConfig) -> Result<()> {
    let url = format!("{}/health", config.api_url);
    let resp = client.get(url).maybe_bearer(&config.token)?.send().await?.error_for_status()?;
    let body = resp.text().await?;
    println!("{body}");
    Ok(())
}

trait AuthRequestBuilder {
    fn maybe_bearer(self, token: &Option<String>) -> Result<Self>
    where
        Self: Sized;
}

impl AuthRequestBuilder for reqwest::RequestBuilder {
    fn maybe_bearer(self, token: &Option<String>) -> Result<Self> {
        if let Some(t) = token {
            Ok(self.bearer_auth(t))
        } else {
            Err(eyre!("Missing API token; run `flora login --token <token>`"))
        }
    }
}

fn collect_files(root: &Path) -> Result<Vec<DeploymentFile>> {
    let mut files = Vec::new();
    collect_files_recursive(root, root, &mut files)?;
    if files.is_empty() {
        return Err(eyre!("No files found under {}", root.display()));
    }
    Ok(files)
}

fn collect_files_recursive(root: &Path, dir: &Path, files: &mut Vec<DeploymentFile>) -> Result<()> {
    for entry in
        fs::read_dir(dir).map_err(|err| eyre!("Failed to read {}: {err}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, files)?;
            continue;
        }
        if !is_allowed_extension(&path) {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|err| eyre!("Failed to read {}: {err}", path.display()))?;
        let rel = path_to_relative(&path, root)?;
        files.push(DeploymentFile { path: rel, contents });
    }
    Ok(())
}

fn is_allowed_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cts")
    )
}

fn path_to_relative(path: &Path, root: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| eyre!("Entry file is not inside {}", root.display()))?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    if rel.is_empty() {
        return Err(eyre!("Entry path is empty"));
    }
    Ok(rel)
}
