use confique::Config;
use std::fmt::Debug;
use std::net::IpAddr;

/// Flora configuration template. To begin, copy this file to `config.toml` and fill in the values.
#[derive(Debug, Config)]
pub struct AppConfig {
    /// Logging config. This is the same as `RUST_LOG`'s format.
    #[config(env = "RUST_LOG", default = "flora::runtime=info,flora=info")]
    pub log_level: String,
    /// Secrets configuration.
    #[config(nested)]
    pub secrets: SecretsConfig,
    /// Discord config.
    #[config(nested)]
    pub discord: DiscordConfig,
    /// Database (PostgreSQL) config.
    #[config(nested)]
    pub database: DatabaseConfig,
    /// Cache (Valkey) config.
    #[config(nested)]
    pub cache: CacheConfig,
    /// Runtime config.
    #[config(nested)]
    pub runtime: RuntimeConfig,
    /// API server config.
    #[config(nested)]
    pub api: ApiConfig,
    /// Build service configuration.
    #[config(nested)]
    pub build_service: BuildServiceConfig,
}

/// Discord OAuth configuration.
#[derive(Debug, Config)]
pub struct DiscordConfig {
    /// Discord bot token.
    #[config(env = "DISCORD_TOKEN")]
    pub bot_token: String,
    /// Discord client ID for OAuth.
    #[config(env = "DISCORD_CLIENT_ID")]
    pub client_id: String,
    /// Discord client secret for OAuth.
    #[config(env = "DISCORD_CLIENT_SECRET")]
    pub client_secret: String,
    /// Discord redirect URI for OAuth. Must match the one in the Discord developer portal, and has to be exposed like so: `https://<host>/auth/callback`
    #[config(
        env = "DISCORD_REDIRECT_URI",
        default = "http://localhost:3000/auth/callback"
    )]
    pub redirect_uri: String,
}

/// Database configuration.
#[derive(Debug, Config)]
pub struct DatabaseConfig {
    /// Database URL.
    #[config(
        env = "DATABASE_URL",
        default = "postgres://user:pass@localhost:5433/flora"
    )]
    pub url: String,
    /// Maximum number of connections to the database.
    #[config(env = "DATABASE_MAX_CONNECTIONS", default = 5)]
    pub max_connections: u32,
}

/// Cache/Valkey configuration.
#[derive(Debug, Config)]
pub struct CacheConfig {
    /// Cache URL.
    #[config(env = "CACHE_URL", default = "redis://127.0.0.1:5434/0")]
    pub url: String,
    /// Pool size.
    #[config(env = "CACHE_POOL_SIZE", default = 10)]
    pub pool_size: usize,
}

/// Runtime configuration.
#[derive(Debug, Config)]
pub struct RuntimeConfig {
    /// Maximum number of worker threads for guild isolates.
    /// Default: 4
    /// Max: 64
    #[config(env = "RUNTIME_MAX_WORKERS", default = 4)]
    pub max_workers: usize,
    /// Command queue capacity per worker.
    /// Default: 1024
    #[config(env = "RUNTIME_WORKER_QUEUE_CAPACITY", default = 1024)]
    pub worker_queue_capacity: usize,
    /// Timeout in seconds for runtime bootstrap (0 disables).
    #[config(env = "RUNTIME_BOOT_TIMEOUT_SECS", default = 5)]
    pub boot_timeout_secs: u64,
    /// Timeout in seconds for script/module load (0 disables).
    #[config(env = "RUNTIME_LOAD_TIMEOUT_SECS", default = 30)]
    pub load_timeout_secs: u64,
    /// Timeout in seconds for per-event dispatch (0 disables).
    #[config(env = "RUNTIME_DISPATCH_TIMEOUT_SECS", default = 3)]
    pub dispatch_timeout_secs: u64,
    /// Timeout in milliseconds for Discord REST requests.
    /// Default: 8000
    #[config(env = "RUNTIME_REST_TIMEOUT_MS", default = 8_000)]
    pub rest_timeout_ms: u64,
    /// Maximum number of concurrent Discord REST requests per guild.
    /// Default: 4
    #[config(env = "RUNTIME_GUILD_CONCURRENCY", default = 4)]
    pub guild_concurrency: usize,
    /// Max script size in bytes (SDK + deployment). Default: 8MB.
    #[config(env = "RUNTIME_MAX_SCRIPT_BYTES", default = 8_388_608)]
    pub max_script_bytes: usize,
    /// Maximum number of deployment files accepted by bundler.
    #[config(env = "RUNTIME_MAX_BUNDLE_FILES", default = 200)]
    pub max_bundle_files: usize,
    /// Maximum total deployment bundle source bytes accepted by bundler.
    #[config(env = "RUNTIME_MAX_BUNDLE_TOTAL_BYTES", default = 1_048_576)]
    pub max_bundle_total_bytes: usize,
    /// Maximum number of cron jobs per guild (or default runtime).
    #[config(env = "RUNTIME_MAX_CRON_JOBS", default = 32)]
    pub max_cron_jobs: usize,
    /// Timeout in seconds for cron handler execution (0 disables).
    #[config(env = "RUNTIME_CRON_TIMEOUT_SECS", default = 5)]
    pub cron_timeout_secs: u64,
    /// Timeout in milliseconds for migration quiesce (0 disables).
    #[config(env = "RUNTIME_MIGRATION_TIMEOUT_MS", default = 500)]
    pub migration_timeout_ms: u64,
    /// Show internal runtime stack frames in user-facing error messages.
    #[config(env = "RUNTIME_SHOW_INTERNAL_STACK_FRAMES", default = false)]
    pub show_internal_stack_frames: bool,
}

/// API server configuration.
#[derive(Debug, Config)]
pub struct ApiConfig {
    /// Port to listen on.
    #[config(env = "API_PORT", default = 3000)]
    pub port: u16,
    /// Bind address.
    #[config(env = "API_ADDRESS", default = "0.0.0.0")]
    pub address: IpAddr,
    /// Secret key for signing cookies.
    #[config(env = "API_SECRET")]
    pub secret: String,
    /// Cookie TTL in seconds. Default is 30 days.
    #[config(env = "API_COOKIE_TTL_SECS", default = 2592000)] // 30 days
    pub cookie_ttl_secs: u64,
    /// Whether to use secure cookies.
    #[config(env = "API_COOKIE_SECURE", default = false)]
    pub cookie_secure: bool,
    /// Optional bearer token for operator-only endpoints such as metrics.
    #[config(env = "API_OPERATOR_SECRET")]
    pub operator_secret: Option<String>,
}

/// Secrets configuration.
#[derive(Debug, Config)]
pub struct SecretsConfig {
    /// 32-byte key for encrypting stored secrets and deriving placeholders.
    /// Example: a 64-char hex string or 32 raw ASCII bytes.
    #[config(env = "SECRETS_MASTER_KEY")]
    pub master_key: String,
}

/// Build service (server-side bundling) configuration.
#[derive(Debug, Config)]
pub struct BuildServiceConfig {
    /// Base URL of the internal build service.
    #[config(env = "BUILD_SERVICE_URL", default = "http://localhost:3001")]
    pub url: String,
    /// Shared secret for authenticating requests to the build service.
    #[config(env = "BUILD_SERVICE_SECRET")]
    pub secret: String,
}

#[cfg(test)]
mod config {
    use super::AppConfig;
    use confique::toml::FormatOptions;

    /// Abuse tests like ts-rs does to generate config.toml.template
    #[test]
    fn generate_config_template() -> Result<(), Box<dyn std::error::Error>> {
        let toml = confique::toml::template::<AppConfig>(FormatOptions::default());
        std::fs::write("../../config.template.toml", toml)?;
        Ok(())
    }
}
