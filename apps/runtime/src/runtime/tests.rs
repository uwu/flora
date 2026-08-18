use std::{
    collections::HashMap,
    net::TcpListener,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use chrono::Utc;
use deno_core::{PollEventLoopOptions, v8};
use fred::prelude::Builder;
use parking_lot::Mutex;
use pg_embed::{
    pg_access::PgAccess,
    pg_enums::PgAuthMethod,
    pg_errors::Error as PgEmbedError,
    pg_fetch::{PG_V13, PgFetchSettings},
    postgres::{PgEmbed, PgSettings},
};
use serde_json::json;
use serenity::{http::Http, secrets::Token};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use super::{
    js::new_js_runtime,
    limits::RuntimeLimits,
    types::{JsRuntimeState, MigrationEnvelope},
    worker::{deploy_guild_to_worker, dispatch_into_runtime, drop_runtime_state},
};
use crate::{
    ops::CronRegistry,
    services::{
        deployments::Deployment,
        discord_rest::{DiscordRest, RestConfig},
        kv::KvService,
        scope_cache::ScopeCache,
        secrets::{SecretService, SecretsRuntimeData},
    },
};

const GUILD_ID: &str = "guild-redeploy";

fn find_open_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("read local addr").port();
    drop(listener);
    port
}

fn test_limits() -> RuntimeLimits {
    RuntimeLimits {
        boot_timeout: None,
        load_timeout: None,
        dispatch_timeout: None,
        cron_timeout: None,
        migration_timeout: None,
        max_script_bytes: 512 * 1024,
        max_cron_jobs: 4,
        show_internal_stack_frames: false,
    }
}

async fn create_embedded_postgres() -> PgEmbed {
    let kv_dir = std::env::temp_dir().join(format!("flora-pg-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&kv_dir).expect("create pg temp dir");

    let settings = PgSettings {
        database_dir: kv_dir,
        port: find_open_port(),
        user: "postgres".to_string(),
        password: "postgres".to_string(),
        auth_method: PgAuthMethod::MD5,
        persistent: false,
        timeout: Some(Duration::from_secs(15)),
        migration_dir: None,
    };
    let fetch_settings = PgFetchSettings {
        version: PG_V13,
        ..Default::default()
    };

    let mut pg = PgEmbed::new(settings, fetch_settings)
        .await
        .expect("create embedded postgres");
    if let Err(err) = pg.setup().await {
        match err {
            PgEmbedError::ReadFileError(_) => {
                PgAccess::purge().await.expect("purge pg-embed cache");
                pg.setup()
                    .await
                    .expect("setup embedded postgres after purge");
            }
            other => {
                panic!("setup embedded postgres: {other}");
            }
        }
    }
    pg.start_db().await.expect("start embedded postgres");
    pg.create_database("flora").await.expect("create test db");
    pg
}

async fn test_kv_service(database_url: &str) -> KvService {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
        .expect("connect pg pool");
    sqlx::query(
        r#"
            CREATE EXTENSION IF NOT EXISTS pgcrypto;
            CREATE TABLE IF NOT EXISTS kv_stores (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                guild_id TEXT NOT NULL,
                store_name TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (guild_id, store_name)
            );
            CREATE INDEX IF NOT EXISTS idx_kv_stores_guild_id ON kv_stores(guild_id);
            "#,
    )
    .execute(&pool)
    .await
    .expect("create kv schema");

    let kv_path = std::env::temp_dir().join(format!("flora-kv-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&kv_path).expect("create kv temp dir");
    KvService::new(pool, kv_path)
}

fn test_discord_rest(http: Arc<Http>) -> Arc<DiscordRest> {
    let cache_config =
        fred::types::config::Config::from_url("redis://localhost:5434").expect("parse cache url");
    let cache_pool = Builder::from_config(cache_config)
        .build_pool(1)
        .expect("create cache pool");
    let scope_cache = ScopeCache::new(http.clone(), cache_pool);
    Arc::new(DiscordRest::new(http, scope_cache, RestConfig::default()))
}

fn test_locker_runtime() -> JsRuntimeState {
    let db = PgPoolOptions::new()
        .connect_lazy("postgres://flora:flora@localhost/flora")
        .expect("create lazy database pool");
    let kv_path = std::env::temp_dir().join(format!("flora-kv-{}", Uuid::new_v4()));
    let kv = KvService::new(db, kv_path);
    let http = Arc::new(Http::new(
        Token::try_from("Bot locker.test.token").expect("token"),
    ));
    let rest = test_discord_rest(http);
    let secrets = Arc::new(SecretsRuntimeData::default());
    let cron_registry = Arc::new(Mutex::new(CronRegistry::new(4)));

    new_js_runtime(
        rest,
        kv,
        secrets,
        Some("locker-test".to_string()),
        cron_registry,
    )
}

#[test]
fn locker_runtime_constructs_with_flora_extensions() {
    crate::v8_init::init();

    thread::spawn(|| {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create Tokio runtime");
        tokio_runtime.block_on(async {
            let mut runtime = test_locker_runtime();
            let result = runtime
                .runtime_mut()
                .execute_script("flora:locker_test", "globalThis.__lockerTest = true;")
                .expect("execute script in Locker runtime");
            drop(result);
            assert!(runtime.runtime_mut().is_idle_for_migration());
            drop_runtime_state(runtime);
        });
    })
    .join()
    .expect("Locker runtime thread panicked");
}

#[test]
fn migration_envelope_moves_runtime_between_os_threads() {
    crate::v8_init::init();

    let (sender, receiver) = mpsc::sync_channel(1);
    let (done_sender, done_receiver) = mpsc::sync_channel(0);
    let source = thread::spawn(move || {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create source Tokio runtime");
        tokio_runtime.block_on(async move {
            let mut runtime = test_locker_runtime();
            let result = runtime
                .runtime_mut()
                .execute_script(
                    "flora:migration_source",
                    "globalThis.__migrationValue = 41;",
                )
                .expect("initialize migration state");
            drop(result);
            assert!(runtime.runtime_mut().is_idle_for_migration());

            let envelope = MigrationEnvelope::new(runtime, Vec::new());
            match sender.send(envelope) {
                Ok(()) => {}
                Err(error) => {
                    drop(error);
                    panic!("destination migration thread stopped");
                }
            }

            done_receiver
                .recv()
                .expect("destination migration thread stopped");
        });
    });

    let destination = thread::spawn(move || {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create destination Tokio runtime");
        tokio_runtime.block_on(async move {
            let envelope = receiver.recv().expect("receive migration envelope");
            let (mut runtime, cron_jobs) = envelope.into_parts();
            assert!(cron_jobs.is_empty());

            runtime
                .runtime_mut()
                .run_event_loop(PollEventLoopOptions::default())
                .await
                .expect("run event loop after migration");
            let result = runtime
                .runtime_mut()
                .execute_script(
                    "flora:migration_destination",
                    "globalThis.__migrationValue + 1",
                )
                .expect("execute after migration");
            let context = runtime.runtime_mut().main_context();
            let value = {
                let mut v8_guard = runtime.runtime_mut().v8_guard();
                v8::scope_with_context!(scope, v8_guard.isolate(), &context);
                let local = v8::Local::new(scope, &result);
                let value = local.int32_value(scope).expect("read migration value");
                drop(result);
                value
            };
            assert_eq!(value, 42);
            drop_runtime_state(runtime);
            done_sender
                .send(())
                .expect("source migration thread stopped");
        });
    });

    source.join().expect("source migration thread panicked");
    destination
        .join()
        .expect("destination migration thread panicked");
}

fn make_deployment(iteration: usize) -> Deployment {
    let bundle = format!(
        "globalThis.__floraDispatch = (event, payload) => {{ globalThis.__last = payload; return payload; }};\n// iteration {iteration}"
    );
    Deployment {
        guild_id: GUILD_ID.to_string(),
        entry: "main.js".to_string(),
        files: None,
        source_map: None,
        bundle,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stress_redeploys_reuse_isolates_without_crash() {
    if std::env::var("FLORA_DB_TESTS").is_err() {
        return;
    }

    let test_future = async {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root")
            .to_path_buf();
        std::env::set_current_dir(&workspace_root).expect("set workspace cwd");

        let mut pg = create_embedded_postgres().await;
        let database_url = pg.full_db_uri("flora");

        let http = Arc::new(Http::new(
            Token::try_from("Bot stress.test.token").expect("token"),
        ));
        let rest = test_discord_rest(http);
        let kv = test_kv_service(&database_url).await;
        let secrets = SecretService::new_for_tests(&database_url).await;
        let limits = test_limits();
        let cron_registry = Arc::new(Mutex::new(CronRegistry::new(limits.max_cron_jobs)));
        let mut guild_runtimes: HashMap<String, JsRuntimeState> = HashMap::new();

        for iteration in 0..10 {
            let deployment = make_deployment(iteration);
            deploy_guild_to_worker(
                &mut guild_runtimes,
                &rest,
                &kv,
                &secrets,
                deployment,
                0,
                &limits,
                cron_registry.clone(),
            )
            .await
            .expect("deploy succeeds");

            let runtime = guild_runtimes
                .get_mut(GUILD_ID)
                .expect("runtime present after deploy");

            dispatch_into_runtime(
                runtime,
                "ping".to_string(),
                json!({ "iteration": iteration }),
                0,
                &limits,
            )
            .await
            .expect("dispatch after deploy");
        }

        assert_eq!(guild_runtimes.len(), 1);
        let runtime = guild_runtimes.remove(GUILD_ID).unwrap();
        drop_runtime_state(runtime);
        pg.stop_db().await.expect("stop embedded postgres");
    };

    tokio::time::timeout(Duration::from_secs(120), test_future)
        .await
        .expect("runtime stress test timed out");
}
