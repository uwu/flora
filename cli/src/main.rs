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
    /// KV store management
    #[command(subcommand)]
    Kv(KvCommands),
}

#[derive(Subcommand, Debug)]
enum KvCommands {
    /// Create a new KV store
    CreateStore {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Store name
        #[arg(long)]
        name: String,
    },
    /// List all KV stores for a guild
    ListStores {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
    },
    /// Delete a KV store
    DeleteStore {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Store name
        #[arg(long)]
        name: String,
    },
    /// Set a key-value pair
    Set {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Store name
        #[arg(long)]
        store: String,
        /// Key
        #[arg(long)]
        key: String,
        /// Value
        value: String,
        /// Optional expiration timestamp (Unix seconds)
        #[arg(long)]
        expiration: Option<i64>,
        /// Optional metadata as JSON string
        #[arg(long)]
        metadata: Option<String>,
    },
    /// Get a value by key
    Get {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Store name
        #[arg(long)]
        store: String,
        /// Key
        key: String,
    },
    /// Delete a key
    Delete {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Store name
        #[arg(long)]
        store: String,
        /// Key
        key: String,
    },
    /// List all keys in a store
    ListKeys {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Store name
        #[arg(long)]
        store: String,
        /// Optional prefix filter
        #[arg(long)]
        prefix: Option<String>,
        /// Maximum keys to return (default 100, max 1000)
        #[arg(long)]
        limit: Option<u32>,
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

#[derive(Serialize)]
struct CreateStoreRequest {
    guild_id: String,
    store_name: String,
}

#[derive(Deserialize)]
struct CreateStoreResponse {
    store: KvStore,
}

#[derive(Deserialize, Debug)]
struct KvStore {
    id: String,
    guild_id: String,
    store_name: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct SetValueRequest {
    value: String,
    expiration: Option<i64>,
    metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct GetValueResponse {
    value: Option<String>,
}

#[derive(Deserialize)]
struct ListKeysResponse {
    keys: Vec<KvKeyInfo>,
    list_complete: bool,
    cursor: Option<String>,
}

#[derive(Deserialize, Debug)]
struct KvKeyInfo {
    name: String,
    expiration: Option<i64>,
    metadata: Option<serde_json::Value>,
}

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
        Commands::Kv(kv_cmd) => handle_kv_command(&client, &config, kv_cmd).await?,
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
    use ignore::WalkBuilder;

    let walker = WalkBuilder::new(root)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .hidden(true)
        .follow_links(false)
        .filter_entry(|entry| {
            let dominated_directory =
                |name| entry.file_type().is_some_and(|ft| ft.is_dir()) && entry.file_name() == name;
            !dominated_directory("node_modules")
                && !dominated_directory("target")
                && !dominated_directory("dist")
                && !dominated_directory(".output")
                && !dominated_directory(".next")
                && !dominated_directory(".nuxt")
                && !dominated_directory(".svelte-kit")
                && !dominated_directory("build")
                && !dominated_directory("out")
                && !dominated_directory(".turbo")
                && !dominated_directory(".cache")
                && !dominated_directory("coverage")
                && !dominated_directory(".parcel-cache")
                && !dominated_directory(".vite")
        })
        .build();

    let mut files = Vec::new();
    for result in walker {
        let entry = result.map_err(|err| eyre!("Failed to walk directory: {err}"))?;
        let path = entry.path();
        if path.is_dir() || !is_allowed_extension(path) {
            continue;
        }
        let contents = fs::read_to_string(path)
            .map_err(|err| eyre!("Failed to read {}: {err}", path.display()))?;
        let rel = path_to_relative(path, root)?;
        files.push(DeploymentFile { path: rel, contents });
    }

    if files.is_empty() {
        return Err(eyre!("No files found under {}", root.display()));
    }
    Ok(files)
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

async fn handle_kv_command(client: &Client, config: &CliConfig, cmd: KvCommands) -> Result<()> {
    match cmd {
        KvCommands::CreateStore { guild, name } => {
            let url = format!("{}/kv/stores", config.api_url);
            let body = CreateStoreRequest { guild_id: guild.clone(), store_name: name.clone() };
            let resp: CreateStoreResponse = client
                .post(url)
                .maybe_bearer(&config.token)?
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            println!(
                "Created KV store '{}' for guild {}",
                resp.store.store_name, resp.store.guild_id
            );
        }
        KvCommands::ListStores { guild } => {
            let url = format!("{}/kv/stores?guild_id={}", config.api_url, guild);
            let stores: Vec<KvStore> = client
                .get(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if stores.is_empty() {
                println!("No KV stores found for guild {}", guild);
            } else {
                println!("KV stores for guild {}:", guild);
                for store in stores {
                    println!("  - {}", store.store_name);
                }
            }
        }
        KvCommands::DeleteStore { guild, name } => {
            let url = format!("{}/kv/stores/{}/{}", config.api_url, guild, name);
            client.delete(url).maybe_bearer(&config.token)?.send().await?.error_for_status()?;
            println!("Deleted KV store '{}' for guild {}", name, guild);
        }
        KvCommands::Set { guild, store, key, value, expiration, metadata } => {
            let url = format!("{}/kv/{}/{}/{}", config.api_url, guild, store, key);
            let metadata_json: Option<serde_json::Value> = match metadata {
                Some(ref m) => Some(
                    serde_json::from_str(m).map_err(|e| eyre!("Invalid JSON metadata: {}", e))?,
                ),
                None => None,
            };
            let body =
                SetValueRequest { value: value.clone(), expiration, metadata: metadata_json };
            client
                .put(url)
                .maybe_bearer(&config.token)?
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
            let mut msg = format!("Set {}={} in store '{}' for guild {}", key, value, store, guild);
            if let Some(exp) = expiration {
                msg.push_str(&format!(" (expires: {})", exp));
            }
            println!("{}", msg);
        }
        KvCommands::Get { guild, store, key } => {
            let url = format!("{}/kv/{}/{}/{}", config.api_url, guild, store, key);
            let resp: GetValueResponse = client
                .get(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            match resp.value {
                Some(value) => println!("{}", value),
                None => println!("Key '{}' not found", key),
            }
        }
        KvCommands::Delete { guild, store, key } => {
            let url = format!("{}/kv/{}/{}/{}", config.api_url, guild, store, key);
            client.delete(url).maybe_bearer(&config.token)?.send().await?.error_for_status()?;
            println!("Deleted key '{}' from store '{}' for guild {}", key, store, guild);
        }
        KvCommands::ListKeys { guild, store, prefix, limit } => {
            let mut url = format!("{}/kv/{}/{}", config.api_url, guild, store);
            let mut params = Vec::new();
            if let Some(p) = prefix {
                params.push(format!("prefix={}", p));
            }
            if let Some(l) = limit {
                params.push(format!("limit={}", l));
            }
            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }
            let resp: ListKeysResponse = client
                .get(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if resp.keys.is_empty() {
                println!("No keys found in store '{}'", store);
            } else {
                println!(
                    "Keys in store '{}' ({} of {} shown):",
                    store,
                    resp.keys.len(),
                    if resp.list_complete { "all" } else { "partial" }
                );
                for key in &resp.keys {
                    let mut line = format!("  - {}", key.name);
                    if let Some(exp) = key.expiration {
                        line.push_str(&format!(" (expires: {})", exp));
                    }
                    if let Some(ref meta) = key.metadata {
                        line.push_str(&format!(" [metadata: {}]", meta));
                    }
                    println!("{}", line);
                }
                if !resp.list_complete {
                    println!(
                        "\nMore keys available. Use --limit {} to get more, or --cursor {} for next page.",
                        limit.unwrap_or(100),
                        resp.cursor.clone().unwrap_or_default()
                    );
                }
            }
        }
    }
    Ok(())
}
