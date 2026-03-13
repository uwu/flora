use color_eyre::eyre::Result;
use confique::Config;
use deno_tls::{rustls, rustls::crypto::CryptoProvider};
use eyre::{Context, eyre};
use flora::{
    discord_handler::DiscordHandler,
    handlers::create_router,
    layers::logger::LoggingMiddleware,
    runtime::BotRuntime,
    services::{
        auth::{AuthConfig, AuthService},
        build::BuildServiceClient,
        deployments::DeploymentService,
        kv::KvService,
        secrets::SecretService,
        tokens::TokenService,
    },
    state::AppState,
    v8_init,
};
use flora_config::AppConfig;
use fred::prelude::*;
use reqwest::StatusCode;
use serenity::all::{Client, GatewayIntents, Token};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use std::{future::IntoFuture, sync::Arc, time::Duration};
use time::macros::format_description;
use tokio::net::TcpListener;
use tower_http::timeout::TimeoutLayer;
use tower_layer::layer_fn;
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::FmtSpan, layer, time::UtcTime},
    prelude::*,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config: AppConfig = Config::builder()
        .env()
        .file("config.toml")
        .load()
        .context("Failed to load config")?;

    // Initialize tracing
    let fmt_layer = layer()
        .with_target(false)
        .with_timer(UtcTime::new(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        )))
        .with_span_events(FmtSpan::FULL)
        .compact();

    let filter_layer = EnvFilter::try_new(&config.log_level)?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    color_eyre::config::HookBuilder::default()
        .issue_url(concat!("https://github.com/uwu/flora", "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .issue_filter(|kind| match kind {
            color_eyre::ErrorKind::NonRecoverable(_) => false,
            color_eyre::ErrorKind::Recoverable(_) => true,
        })
        .install()?;

    CryptoProvider::install_default(rustls::crypto::ring::default_provider()).ok();

    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
    MIGRATOR.run(&db_pool).await?;

    let cache_config: fred::types::config::Config =
        fred::types::config::Config::from_url(&config.cache.url)?;
    let cache_client = Builder::from_config(cache_config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(10);
        })
        // use exponential backoff, starting at 100 ms and doubling on each failed attempt up to 30 sec
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(config.cache.pool_size)
        .expect("Failed to create redis pool");

    let cache_task = cache_client
        .init()
        .await
        .expect("Failed to connect to redis");

    let deployment_service =
        DeploymentService::new(db_pool.clone(), cache_client.clone(), cache_task);
    let token_service = TokenService::new(db_pool.clone());
    let kv_service = KvService::new(db_pool.clone(), "./data/kv".into());
    let secret_service = SecretService::new(db_pool.clone(), config.secrets.master_key.clone())?;
    let auth_task = cache_client.clone().init().await?;
    let auth_service = AuthService::new(
        AuthConfig {
            client_id: config.discord.client_id,
            client_secret: config.discord.client_secret,
            redirect_uri: config.discord.redirect_uri,
            session_secret: config.api.secret,
            session_ttl_secs: config.api.cookie_ttl_secs,
            cookie_secure: config.api.cookie_secure,
        },
        cache_client.clone(),
        auth_task,
    )?;
    let build_service =
        BuildServiceClient::new(config.build_service.url, config.build_service.secret)?;

    v8_init::init();

    let token: Token = config
        .discord
        .bot_token
        .parse()
        .map_err(|err: serenity::secrets::TokenError| eyre!(err))?;
    let http = Arc::new(serenity::http::Http::new(token.clone()));

    // Set application id early so guild command registration works before the READY event fires.
    let app_info = http.get_current_application_info().await?;
    http.set_application_id(app_info.id);

    let runtime = Arc::new(BotRuntime::new(
        http.clone(),
        kv_service.clone(),
        secret_service.clone(),
        config.runtime,
        cache_client.clone(),
    ));
    runtime.initialize().await.map_err(|err| eyre!(err))?;

    if let Err(err) = runtime
        .load_sdk_bundle("runtime-dist/runtime_sdk_bundle.js")
        .await
    {
        error!("Failed to load SDK bundle: {:?}", err);
    }

    let cached_deployments = deployment_service.list_deployments().await?;
    for deployment in cached_deployments {
        if let Err(err) = runtime.deploy_guild_script(deployment.clone()).await {
            error!(
                "Failed to load deployment for guild {}: {:?}",
                deployment.guild_id, err
            );
        }
    }

    let intents = GatewayIntents::all();

    let handler = Arc::new(DiscordHandler {
        runtime: runtime.clone(),
        rest: runtime.discord_rest(),
        http: http.clone(),
        application_id: Arc::new(std::sync::RwLock::new(Some(app_info.id))),
        deployments: deployment_service.clone(),
    });

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await?;

    let api_state = AppState {
        runtime: runtime.clone(),
        deployments: deployment_service.clone(),
        auth: auth_service.clone(),
        tokens: token_service.clone(),
        kv: kv_service.clone(),
        secrets: secret_service.clone(),
        build_service,
        http: http.clone(),
    };

    let api_router = create_router()
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(layer_fn(LoggingMiddleware))
        .with_state(api_state);

    let listener = TcpListener::bind(format!("{}:{}", config.api.address, config.api.port))
        .await
        .with_context(|| {
            format!(
                "Failed to bind to {}:{}",
                config.api.address, config.api.port
            )
        })?;

    let api_service = api_router.into_make_service();
    let api_task = tokio::spawn(axum::serve(listener, api_service).into_future());

    let discord_task = tokio::spawn(async move { client.start().await });

    let (api_res, discord_res) = tokio::try_join!(api_task, discord_task)?;
    api_res.map_err(|err: std::io::Error| eyre!(err))?;
    discord_res.map_err(|err: serenity::Error| eyre!(err))?;

    Ok(())
}
