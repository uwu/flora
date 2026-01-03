mod auth;
mod bundler;
mod deployments;
mod discord_handler;
mod handlers;
mod kv;
mod log_sink;
mod metrics;
mod ops;
mod runtime;
mod state;
mod tokens;
mod transpile;
mod v8_init;

use std::{future::IntoFuture, net::SocketAddr, path::Path, sync::Arc};

use auth::{AuthConfig, AuthService};
use color_eyre::eyre::Result;
use deno_tls::{rustls, rustls::crypto::CryptoProvider};
use deployments::DeploymentService;
use discord_handler::DiscordHandler;
use eyre::eyre;
use fred::prelude::*;
use handlers::create_router;
use kv::KvService;
use runtime::BotRuntime;
use serenity::all::{Client, GatewayIntents};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use state::AppState;
use tokens::TokenService;
use tokio::net::TcpListener;
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    CryptoProvider::install_default(rustls::crypto::ring::default_provider()).ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| color_eyre::eyre::eyre!("DISCORD_TOKEN environment variable not set"))?;
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5433/flora".to_string());
    let valkey_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:5434/0".to_string());
    let discord_client_id =
        std::env::var("DISCORD_CLIENT_ID").map_err(|_| eyre!("DISCORD_CLIENT_ID not set"))?;
    let discord_client_secret = std::env::var("DISCORD_CLIENT_SECRET")
        .map_err(|_| eyre!("DISCORD_CLIENT_SECRET not set"))?;
    let discord_redirect_uri = std::env::var("DISCORD_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
    let session_secret =
        std::env::var("SESSION_SECRET").map_err(|_| eyre!("SESSION_SECRET not set"))?;
    let session_ttl_secs = std::env::var("SESSION_TTL_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(60 * 60 * 24 * 30);
    let cookie_secure = std::env::var("COOKIE_SECURE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or_else(|| discord_redirect_uri.starts_with("https://"));
    let api_addr: SocketAddr = std::env::var("API_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .map_err(|_| eyre!("invalid API_ADDR"))?;

    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;
    static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
    MIGRATOR.run(&pool).await?;

    let valkey_config = Config::from_url(&valkey_url)?;
    let valkey_client = Builder::from_config(valkey_config).build()?;
    let valkey_task = valkey_client.init().await?;
    let deployment_service =
        DeploymentService::new(pool.clone(), valkey_client.clone(), valkey_task);
    let token_service = TokenService::new(pool.clone());
    let kv_service = KvService::new(pool.clone(), "./data/kv".into());
    let auth_task = valkey_client.clone().init().await?;
    let auth_service = AuthService::new(
        AuthConfig {
            client_id: discord_client_id,
            client_secret: discord_client_secret,
            redirect_uri: discord_redirect_uri,
            session_secret,
            session_ttl_secs,
            cookie_secure,
        },
        valkey_client.clone(),
        auth_task,
    )?;

    v8_init::init();

    let http = Arc::new(serenity::http::Http::new(&token));

    // Set application id early so guild command registration works before the READY event fires.
    let app_info = http.get_current_application_info().await?;
    http.set_application_id(app_info.id);

    let runtime = Arc::new(BotRuntime::new(http.clone(), kv_service.clone()));
    runtime.initialize().await.map_err(|err| eyre!(err))?;

    if let Err(err) = runtime.load_user_script("dist/sdk-bundle.js").await {
        error!("Failed to load SDK bundle: {:?}", err);
    }

    // Optionally load a default script for local development when present.
    if Path::new("scripts/bot.ts").exists() {
        if let Err(err) = runtime.load_user_script("scripts/bot.ts").await {
            error!("Failed to load user script: {:?}", err);
        }
    }

    let cached_deployments = deployment_service.list_deployments().await?;
    for deployment in cached_deployments {
        if let Err(err) = runtime.deploy_guild_script(deployment.clone()).await {
            error!("Failed to load deployment for guild {}: {:?}", deployment.guild_id, err);
        }
    }

    let intents = GatewayIntents::all();

    let handler = DiscordHandler {
        runtime: runtime.clone(),
        http: http.clone(),
        application_id: Arc::new(std::sync::RwLock::new(Some(app_info.id))),
        deployments: deployment_service.clone(),
    };

    let mut client = Client::builder(&token, intents).event_handler(handler).await?;

    let api_state = AppState {
        runtime: runtime.clone(),
        deployments: deployment_service.clone(),
        auth: auth_service.clone(),
        tokens: token_service.clone(),
        kv: kv_service.clone(),
        http: http.clone(),
    };

    let api_router = create_router().with_state(api_state);
    let listener = TcpListener::bind(api_addr).await?;
    let api_service = api_router.into_make_service();
    let api_task = tokio::spawn(axum::serve(listener, api_service).into_future());

    let discord_task = tokio::spawn(async move { client.start().await });

    let (api_res, discord_res) = tokio::try_join!(api_task, discord_task)?;
    api_res.map_err(|err: std::io::Error| eyre!(err))?;
    discord_res.map_err(|err: serenity::Error| eyre!(err))?;

    Ok(())
}
