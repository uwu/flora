use std::sync::Arc;

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use fred::{prelude::*, types::ConnectHandle};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::bundler::DeploymentFile;

/// Stored representation of a deployment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deployment {
    pub scope_type: String,
    pub scope_id: String,
    pub entry: String,
    pub files: Vec<DeploymentFile>,
    pub bundle: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DeploymentService {
    db_pool: Pool<Postgres>,
    cache_pool: fred::prelude::Pool,
    _cache_task: Arc<ConnectHandle>,
}

#[derive(FromRow)]
struct DeploymentRow {
    scope_type: String,
    scope_id: String,
    entry: String,
    files: sqlx::types::Json<Vec<DeploymentFile>>,
    bundle: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DeploymentService {
    pub fn new(
        db_pool: Pool<Postgres>,
        cache_pool: fred::prelude::Pool,
        cache_task: ConnectHandle,
    ) -> Self {
        Self {
            db_pool,
            cache_pool,
            _cache_task: Arc::new(cache_task),
        }
    }

    pub async fn upsert_deployment(
        &self,
        scope_type: String,
        scope_id: String,
        entry: String,
        files: Vec<DeploymentFile>,
        bundle: String,
    ) -> Result<Deployment> {
        let record = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deployments (scope_type, scope_id, entry, files, bundle)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (scope_type, scope_id) DO UPDATE
            SET entry = EXCLUDED.entry,
                files = EXCLUDED.files,
                bundle = EXCLUDED.bundle,
                updated_at = NOW()
            RETURNING scope_type, scope_id, entry, files, bundle, created_at, updated_at
            "#,
        )
        .bind(&scope_type)
        .bind(&scope_id)
        .bind(&entry)
        .bind(sqlx::types::Json(&files))
        .bind(&bundle)
        .fetch_one(&self.db_pool)
        .await?;

        let deployment = to_deployment(record)?;
        self.cache_deployment(&deployment).await?;
        info!(target: "flora:deployments", scope_type = deployment.scope_type, scope_id = deployment.scope_id, "deployment stored");
        Ok(deployment)
    }

    pub async fn get_deployment(
        &self,
        scope_type: &str,
        scope_id: &str,
    ) -> Result<Option<Deployment>> {
        if let Some(cached) = self.fetch_cached_deployment(scope_type, scope_id).await? {
            return Ok(Some(cached));
        }

        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT scope_type, scope_id, entry, files, bundle, created_at, updated_at
            FROM deployments
            WHERE scope_type = $1 AND scope_id = $2
            "#,
        )
        .bind(scope_type)
        .bind(scope_id)
        .fetch_optional(&self.db_pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let deployment = to_deployment(row)?;
        self.cache_deployment(&deployment).await?;
        Ok(Some(deployment))
    }

    pub async fn delete_deployment(&self, scope_type: &str, scope_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM deployments WHERE scope_type = $1 AND scope_id = $2")
            .bind(scope_type)
            .bind(scope_id)
            .execute(&self.db_pool)
            .await?;
        let key = cache_key(scope_type, scope_id);
        let _: () = self.cache_pool.del(key).await.unwrap_or_default();
        info!(target: "flora:deployments", scope_type, scope_id, "deployment deleted");
        Ok(())
    }

    pub async fn list_deployments(&self) -> Result<Vec<Deployment>> {
        let rows = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT scope_type, scope_id, entry, files, bundle, created_at, updated_at
            FROM deployments
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut deployments = Vec::new();
        for row in rows {
            deployments.push(to_deployment(row)?);
        }
        Ok(deployments)
    }

    async fn cache_deployment(&self, deployment: &Deployment) -> Result<()> {
        let key = cache_key(&deployment.scope_type, &deployment.scope_id);
        let value = serde_json::to_string(deployment)?;
        // cache for 10 minutes to reduce DB traffic.
        self.cache_pool
            .set::<(), _, _>(key, value, Some(Expiration::EX(600)), None, false)
            .await?;
        Ok(())
    }

    async fn fetch_cached_deployment(
        &self,
        scope_type: &str,
        scope_id: &str,
    ) -> Result<Option<Deployment>> {
        let key = cache_key(scope_type, scope_id);
        let value: Option<String> = self.cache_pool.get(key).await?;
        if let Some(raw) = value {
            match serde_json::from_str::<Deployment>(&raw) {
                Ok(deployment) => Ok(Some(deployment)),
                Err(err) => {
                    warn!(target: "flora:deployments", scope_type, scope_id, ?err, "failed to decode cached deployment");
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

fn cache_key(scope_type: &str, scope_id: &str) -> String {
    format!("deployment:{scope_type}:{scope_id}")
}

fn to_deployment(row: DeploymentRow) -> Result<Deployment> {
    let DeploymentRow {
        scope_type,
        scope_id,
        entry,
        files,
        bundle,
        created_at,
        updated_at,
    } = row;
    Ok(Deployment {
        scope_type,
        scope_id,
        entry,
        files: files.0,
        bundle,
        created_at,
        updated_at,
    })
}

impl Deployment {
    /// Derive a synthetic module name for bundled modules.
    pub fn module_name(&self) -> String {
        format!("{}:{}.bundle.js", self.scope_type, self.scope_id)
    }

    pub fn scope_key(&self) -> String {
        if self.scope_type == "guild" {
            self.scope_id.clone()
        } else {
            format!("{}:{}", self.scope_type, self.scope_id)
        }
    }
}
