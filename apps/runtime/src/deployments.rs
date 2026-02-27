use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use fred::{prelude::*, types::ConnectHandle};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use std::sync::Arc;
use tracing::{info, warn};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentSourceMapFile {
    pub path: String,
    pub contents: String,
}

/// Stored representation of a guild deployment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deployment {
    pub guild_id: String,
    pub entry: String,
    pub source_map: Option<DeploymentSourceMapFile>,
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
    guild_id: String,
    entry: String,
    source_map: Option<sqlx::types::Json<DeploymentSourceMapFile>>,
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
        guild_id: String,
        entry: String,
        bundle: String,
        source_map: Option<DeploymentSourceMapFile>,
    ) -> Result<Deployment> {
        let record = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deployments (guild_id, entry, bundle, source_map)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id) DO UPDATE
            SET entry = EXCLUDED.entry,
                bundle = EXCLUDED.bundle,
                source_map = EXCLUDED.source_map,
                updated_at = NOW()
            RETURNING guild_id, entry, source_map, bundle, created_at, updated_at
            "#,
        )
        .bind(&guild_id)
        .bind(&entry)
        .bind(&bundle)
        .bind(source_map.map(sqlx::types::Json))
        .fetch_one(&self.db_pool)
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
            SELECT guild_id, entry, source_map, bundle, created_at, updated_at
            FROM deployments
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db_pool)
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
            SELECT guild_id, entry, source_map, bundle, created_at, updated_at
            FROM deployments
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        rows.into_iter()
            .map(to_deployment)
            .collect::<Result<Vec<_>>>()
    }

    async fn cache_deployment(&self, deployment: &Deployment) -> Result<()> {
        let key = cache_key(&deployment.guild_id);
        let value = serde_json::to_string(deployment)?;
        // cache for 10 minutes to reduce DB traffic.
        self.cache_pool
            .set::<(), _, _>(key, value, Some(Expiration::EX(600)), None, false)
            .await?;
        Ok(())
    }

    async fn fetch_cached_deployment(&self, guild_id: &str) -> Result<Option<Deployment>> {
        let key = cache_key(guild_id);
        let value: Option<String> = self.cache_pool.get(key).await?;
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
        source_map: row.source_map.map(|source_map| source_map.0),
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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{Deployment, DeploymentSourceMapFile};

    #[test]
    fn deployment_json_roundtrip_preserves_source_map() {
        let deployment = Deployment {
            guild_id: "123".to_string(),
            entry: "src/main.ts".to_string(),
            source_map: Some(DeploymentSourceMapFile {
                path: "bundle.js.map".to_string(),
                contents: "{\"version\":3}".to_string(),
            }),
            bundle: "console.log('hi')".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let encoded = serde_json::to_string(&deployment).expect("serialize deployment");
        let decoded: Deployment = serde_json::from_str(&encoded).expect("deserialize deployment");

        assert_eq!(
            decoded.source_map.expect("source map").path,
            "bundle.js.map"
        );
    }

    #[test]
    fn deployment_json_allows_missing_source_map() {
        let value = json!({
            "guild_id": "123",
            "entry": "src/main.ts",
            "bundle": "console.log('hi')",
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": Utc::now().to_rfc3339()
        });

        let deployment: Deployment =
            serde_json::from_value(value).expect("deserialize deployment without source map");

        assert!(deployment.source_map.is_none());
    }
}
