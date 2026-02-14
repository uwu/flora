use super::{
    limits::RuntimeLimits,
    types::JsRuntimeState,
    worker::{deploy_guild_to_worker, dispatch_into_runtime, drop_runtime_state},
};
use crate::{deployments::Deployment, kv::KvService, ops::CronRegistry, secrets::SecretService};
use chrono::Utc;
use parking_lot::Mutex;
use serde_json::json;
use serenity::{http::Http, secrets::Token};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, net::TcpListener, path::PathBuf, sync::Arc, time::Duration};
use uuid::Uuid;

use pg_embed::{
    pg_access::PgAccess,
    pg_enums::PgAuthMethod,
    pg_errors::PgEmbedErrorType,
    pg_fetch::{PG_V13, PgFetchSettings},
    postgres::{PgEmbed, PgSettings},
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
        match err.error_type {
            PgEmbedErrorType::ReadFileError => {
                PgAccess::purge().await.expect("purge pg-embed cache");
                pg.setup()
                    .await
                    .expect("setup embedded postgres after purge");
            }
            _ => {
                panic!("setup embedded postgres: {err}");
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

fn make_deployment(iteration: usize) -> Deployment {
    let bundle = format!(
        "globalThis.__floraDispatch = (event, payload) => {{ globalThis.__last = payload; return payload; }};\n// iteration {iteration}"
    );
    Deployment {
        guild_id: GUILD_ID.to_string(),
        entry: "main.js".to_string(),
        files: Vec::new(),
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
        let kv = test_kv_service(&database_url).await;
        let secrets = SecretService::new_for_tests(&database_url).await;
        let limits = test_limits();
        let cron_registry = Arc::new(Mutex::new(CronRegistry::new(limits.max_cron_jobs)));
        let mut guild_runtimes: HashMap<String, JsRuntimeState> = HashMap::new();

        for iteration in 0..10 {
            let deployment = make_deployment(iteration);
            deploy_guild_to_worker(
                &mut guild_runtimes,
                &http,
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
