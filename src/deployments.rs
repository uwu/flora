use std::sync::Arc;

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use fred::{prelude::*, types::ConnectHandle};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::bundler::DeploymentFile;

/// Stored representation of a guild deployment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deployment {
    pub guild_id: String,
    pub entry: String,
    pub files: Vec<DeploymentFile>,
    pub bundle: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DeploymentService {
    db: Pool<Postgres>,
    cache: Client,
    _cache_task: Arc<ConnectHandle>,
}

#[derive(FromRow)]
struct DeploymentRow {
    guild_id: String,
    entry: String,
    files: sqlx::types::Json<Vec<DeploymentFile>>,
    bundle: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DeploymentService {
    pub fn new(db: Pool<Postgres>, cache: Client, cache_task: ConnectHandle) -> Self {
        Self { db, cache, _cache_task: Arc::new(cache_task) }
    }

    pub async fn upsert_deployment(
        &self,
        guild_id: String,
        entry: String,
        files: Vec<DeploymentFile>,
        bundle: String,
    ) -> Result<Deployment> {
        let record = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deployments (guild_id, entry, files, bundle)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id) DO UPDATE
            SET entry = EXCLUDED.entry,
                files = EXCLUDED.files,
                bundle = EXCLUDED.bundle,
                updated_at = NOW()
            RETURNING guild_id, entry, files, bundle, created_at, updated_at
            "#,
        )
        .bind(&guild_id)
        .bind(&entry)
        .bind(sqlx::types::Json(&files))
        .bind(&bundle)
        .fetch_one(&self.db)
        .await?;

        let deployment = to_deployment(record)?;
        self.cache_deployment(&deployment).await?;
        info!(target: "flora:deployments", guild_id = deployment.guild_id, "deployment stored");
        Ok(deployment)
    }

    pub async fn get_deployment(&self, guild_id: &str) -> Result<Option<Deployment>> {
        if let Some(cached) = self.fetch_cached_deployment(guild_id).await? {
            return Ok(Some(cached));
        }

        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT guild_id, entry, files, bundle, created_at, updated_at
            FROM deployments
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let deployment = to_deployment(row)?;
        self.cache_deployment(&deployment).await?;
        Ok(Some(deployment))
    }

    pub async fn list_deployments(&self) -> Result<Vec<Deployment>> {
        let rows = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT guild_id, entry, files, bundle, created_at, updated_at
            FROM deployments
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(to_deployment).collect::<Result<Vec<_>>>()
    }

    async fn cache_deployment(&self, deployment: &Deployment) -> Result<()> {
        let key = cache_key(&deployment.guild_id);
        let value = serde_json::to_string(deployment)?;
        // cache for 10 minutes to reduce DB traffic.
        self.cache.set::<(), _, _>(key, value, Some(Expiration::EX(600)), None, false).await?;
        Ok(())
    }

    async fn fetch_cached_deployment(&self, guild_id: &str) -> Result<Option<Deployment>> {
        let key = cache_key(guild_id);
        let value: Option<String> = self.cache.get(key).await?;
        if let Some(raw) = value {
            match serde_json::from_str::<Deployment>(&raw) {
                Ok(deployment) => Ok(Some(deployment)),
                Err(err) => {
                    warn!(target: "flora:deployments", guild_id, ?err, "failed to decode cached deployment");
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

fn cache_key(guild_id: &str) -> String {
    format!("deployment:{guild_id}")
}

fn to_deployment(row: DeploymentRow) -> Result<Deployment> {
    Ok(Deployment {
        guild_id: row.guild_id,
        entry: row.entry,
        files: row.files.0,
        bundle: row.bundle,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

impl Deployment {
    /// Derive a synthetic module name for bundled modules.
    pub fn module_name(&self) -> String {
        format!("guild:{}.bundle.js", self.guild_id)
    }
}
