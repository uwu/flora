# Flora Configuration

Centralized configuration management for the Flora bot application using [confique](https://docs.rs/confique/latest/confique/) with TOML support.

## Features

- **TOML-based configuration** (`flora.toml`)
- **Environment variable fallback** for all settings
- **Typed configuration** with serde integration
- **Sensible defaults** for all optional settings
- **Builder API** for flexible configuration loading

## Configuration Structure

```toml
[discord]
bot_token = "..."
client_id = "..."
client_secret = "..."
redirect_uri = "http://localhost:3000/auth/callback"  # optional

[database]
url = "postgres://user:pass@localhost:5433/flora"     # optional
max_connections = 5                                    # optional

[cache]
url = "redis://127.0.0.1:5434/0"                      # optional

[session]
secret = "your-secret-here-min-32-chars"
ttl_secs = 2592000                                     # optional (30 days default)
cookie_secure = true                                   # optional

[runtime]
max_workers = 4                                        # optional (defaults to min(num_cpus, 4))

[api]
addr = "0.0.0.0:3000"                                 # optional
```

## Environment Variables

All configuration fields can be set via environment variables. The format follows the section structure:

- `DISCORD_BOT_TOKEN`
- `DISCORD_CLIENT_ID`
- `DISCORD_CLIENT_SECRET`
- `DISCORD_REDIRECT_URI`
- `DATABASE_URL`
- `DATABASE_MAX_CONNECTIONS`
- `CACHE_URL`
- `SESSION_SECRET`
- `SESSION_TTL_SECS`
- `SESSION_COOKIE_SECURE`
- `RUNTIME_MAX_WORKERS`
- `API_ADDR`

## Usage

### Loading Configuration

```rust
use flora_config::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load from flora.toml and environment
    let config = AppConfig::load()?;
    
    // Access config
    println!("Bot token: {}", config.discord.bot_token);
    println!("Database: {}", config.database.url);
    
    Ok(())
}
```

### Builder API

```rust
use flora_config::AppConfig;

let config = AppConfig::builder()
    .env()
    .file("flora.toml")
    .load()?;
```

## Migration Guide

See the root directory's migration example or run the main application with `RUST_LOG=flora_config=debug` to see configuration loading details.
