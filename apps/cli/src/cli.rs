use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "flora", about = "Deployment CLI for flora guild scripts")]
pub(crate) struct Cli {
    /// API base URL (env: FLORA_API_URL). Overrides config.
    #[arg(long, env = "FLORA_API_URL")]
    pub(crate) api_url: Option<String>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
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
    /// View runtime logs
    Logs {
        /// Discord guild ID to filter logs (optional)
        #[arg(long)]
        guild: Option<String>,
        /// Follow logs in real-time (stream via SSE)
        #[arg(short, long)]
        follow: bool,
        /// Maximum number of log entries to fetch (default 100, max 1000)
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },
    /// KV store management
    #[command(subcommand)]
    Kv(KvCommands),
}

#[derive(Subcommand, Debug)]
pub(crate) enum KvCommands {
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
