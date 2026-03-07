use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use fred::{prelude::*, types::ConnectHandle};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tracing::{info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::bundler::DeploymentFile;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentSourceMapFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentChangeSummary {
    pub added_files: usize,
    pub removed_files: usize,
    pub modified_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentRevisionStatus {
    Success,
    Failed,
}

impl DeploymentRevisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for DeploymentRevisionStatus {
    type Err = color_eyre::eyre::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            _ => Err(eyre!("invalid deployment revision status: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentSource {
    Cli,
    Webui,
    Bootstrap,
    Api,
    Unknown,
}

impl DeploymentSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Webui => "webui",
            Self::Bootstrap => "bootstrap",
            Self::Api => "api",
            Self::Unknown => "unknown",
        }
    }
}

impl FromStr for DeploymentSource {
    type Err = color_eyre::eyre::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "cli" => Ok(Self::Cli),
            "webui" => Ok(Self::Webui),
            "bootstrap" => Ok(Self::Bootstrap),
            "api" => Ok(Self::Api),
            "unknown" => Ok(Self::Unknown),
            _ => Err(eyre!("invalid deployment source: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentActorType {
    Session,
    Token,
    System,
}

impl DeploymentActorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Token => "token",
            Self::System => "system",
        }
    }
}

impl FromStr for DeploymentActorType {
    type Err = color_eyre::eyre::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "session" => Ok(Self::Session),
            "token" => Ok(Self::Token),
            "system" => Ok(Self::System),
            _ => Err(eyre!("invalid deployment actor type: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentRevision {
    pub id: Uuid,
    pub guild_id: String,
    pub entry: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
    pub bundle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
    pub status: DeploymentRevisionStatus,
    pub deployed_at: DateTime<Utc>,
    pub deploy_source: DeploymentSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_username: Option<String>,
    pub actor_type: DeploymentActorType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_revision_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_summary: Option<DeploymentChangeSummary>,
}

#[derive(Debug, Clone)]
pub struct CreateDeploymentRevisionInput {
    pub guild_id: String,
    pub entry: String,
    pub files: Option<Vec<DeploymentFile>>,
    pub bundle: String,
    pub source_map: Option<DeploymentSourceMapFile>,
    pub status: DeploymentRevisionStatus,
    pub deploy_source: DeploymentSource,
    pub actor_user_id: Option<String>,
    pub actor_username: Option<String>,
    pub actor_type: DeploymentActorType,
    pub error_message: Option<String>,
    pub build_id: Option<String>,
    pub base_revision_id: Option<Uuid>,
    pub change_summary: Option<DeploymentChangeSummary>,
}

#[derive(Debug, Clone)]
pub struct DeploymentRevisionCursor {
    pub deployed_at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PreviousSuccessfulRevision {
    pub revision_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
}

/// Stored representation of a guild deployment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deployment {
    pub guild_id: String,
    pub entry: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
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
    files: Option<sqlx::types::Json<Vec<DeploymentFile>>>,
    source_map: Option<sqlx::types::Json<DeploymentSourceMapFile>>,
    bundle: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct DeploymentRevisionRow {
    id: Uuid,
    guild_id: String,
    entry: String,
    files: Option<sqlx::types::Json<Vec<DeploymentFile>>>,
    bundle: String,
    source_map: Option<sqlx::types::Json<DeploymentSourceMapFile>>,
    status: String,
    deployed_at: DateTime<Utc>,
    deploy_source: String,
    actor_user_id: Option<String>,
    actor_username: Option<String>,
    actor_type: String,
    error_message: Option<String>,
    build_id: Option<String>,
    base_revision_id: Option<Uuid>,
    change_summary: Option<sqlx::types::Json<DeploymentChangeSummary>>,
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
        files: Option<Vec<DeploymentFile>>,
        bundle: String,
        source_map: Option<DeploymentSourceMapFile>,
    ) -> Result<Deployment> {
        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deployments (guild_id, entry, files, bundle, source_map)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (guild_id) DO UPDATE
            SET entry = EXCLUDED.entry,
                files = EXCLUDED.files,
                bundle = EXCLUDED.bundle,
                source_map = EXCLUDED.source_map,
                updated_at = NOW()
            RETURNING guild_id, entry, files, source_map, bundle, created_at, updated_at
            "#,
        )
        .bind(&guild_id)
        .bind(&entry)
        .bind(files.map(sqlx::types::Json))
        .bind(&bundle)
        .bind(source_map.map(sqlx::types::Json))
        .fetch_one(&self.db_pool)
        .await?;

        let deployment = to_deployment(row)?;
        self.cache_deployment(&deployment).await?;
        info!(target: "flora:deployments", guild_id = deployment.guild_id, "deployment stored");
        Ok(deployment)
    }

    pub async fn create_revision(
        &self,
        input: CreateDeploymentRevisionInput,
    ) -> Result<DeploymentRevision> {
        let row = sqlx::query_as::<_, DeploymentRevisionRow>(
            r#"
            INSERT INTO deployment_revisions (
                guild_id, entry, files, bundle, source_map, status, deploy_source,
                actor_user_id, actor_username, actor_type, error_message, build_id,
                base_revision_id, change_summary
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, guild_id, entry, files, bundle, source_map, status, deployed_at,
                deploy_source, actor_user_id, actor_username, actor_type, error_message,
                build_id, base_revision_id, change_summary
            "#,
        )
        .bind(&input.guild_id)
        .bind(&input.entry)
        .bind(input.files.map(sqlx::types::Json))
        .bind(&input.bundle)
        .bind(input.source_map.map(sqlx::types::Json))
        .bind(input.status.as_str())
        .bind(input.deploy_source.as_str())
        .bind(input.actor_user_id)
        .bind(input.actor_username)
        .bind(input.actor_type.as_str())
        .bind(input.error_message)
        .bind(input.build_id)
        .bind(input.base_revision_id)
        .bind(input.change_summary.map(sqlx::types::Json))
        .fetch_one(&self.db_pool)
        .await?;

        to_revision(row)
    }

    pub async fn list_guild_revisions(
        &self,
        guild_id: &str,
        limit: i64,
        cursor: Option<DeploymentRevisionCursor>,
    ) -> Result<Vec<DeploymentRevision>> {
        let rows = if let Some(cursor) = cursor {
            sqlx::query_as::<_, DeploymentRevisionRow>(
                r#"
                SELECT id, guild_id, entry, files, bundle, source_map, status, deployed_at,
                    deploy_source, actor_user_id, actor_username, actor_type, error_message,
                    build_id, base_revision_id, change_summary
                FROM deployment_revisions
                WHERE guild_id = $1
                  AND (deployed_at, id) < ($2, $3)
                ORDER BY deployed_at DESC, id DESC
                LIMIT $4
                "#,
            )
            .bind(guild_id)
            .bind(cursor.deployed_at)
            .bind(cursor.id)
            .bind(limit)
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query_as::<_, DeploymentRevisionRow>(
                r#"
                SELECT id, guild_id, entry, files, bundle, source_map, status, deployed_at,
                    deploy_source, actor_user_id, actor_username, actor_type, error_message,
                    build_id, base_revision_id, change_summary
                FROM deployment_revisions
                WHERE guild_id = $1
                ORDER BY deployed_at DESC, id DESC
                LIMIT $2
                "#,
            )
            .bind(guild_id)
            .bind(limit)
            .fetch_all(&self.db_pool)
            .await?
        };

        let mut revisions = Vec::with_capacity(rows.len());
        for row in rows {
            revisions.push(to_revision(row)?);
        }
        Ok(revisions)
    }

    pub async fn get_guild_revision(
        &self,
        guild_id: &str,
        revision_id: Uuid,
    ) -> Result<Option<DeploymentRevision>> {
        let row = sqlx::query_as::<_, DeploymentRevisionRow>(
            r#"
            SELECT id, guild_id, entry, files, bundle, source_map, status, deployed_at,
                deploy_source, actor_user_id, actor_username, actor_type, error_message,
                build_id, base_revision_id, change_summary
            FROM deployment_revisions
            WHERE guild_id = $1 AND id = $2
            "#,
        )
        .bind(guild_id)
        .bind(revision_id)
        .fetch_optional(&self.db_pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(to_revision(row)?))
    }

    pub async fn get_current_successful(&self, guild_id: &str) -> Result<Option<Deployment>> {
        let row = sqlx::query_as::<_, DeploymentRevisionRow>(
            r#"
            SELECT id, guild_id, entry, files, bundle, source_map, status, deployed_at,
                deploy_source, actor_user_id, actor_username, actor_type, error_message,
                build_id, base_revision_id, change_summary
            FROM deployment_revisions
            WHERE guild_id = $1 AND status = 'success'
            ORDER BY deployed_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db_pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let revision = to_revision(row)?;
        Ok(Some(Deployment {
            guild_id: revision.guild_id,
            entry: revision.entry,
            files: revision.files,
            source_map: revision.source_map,
            bundle: revision.bundle,
            created_at: revision.deployed_at,
            updated_at: revision.deployed_at,
        }))
    }

    pub async fn get_previous_successful_revision(
        &self,
        guild_id: &str,
    ) -> Result<Option<PreviousSuccessfulRevision>> {
        let row = sqlx::query_as::<_, DeploymentRevisionRow>(
            r#"
            SELECT id, guild_id, entry, files, bundle, source_map, status, deployed_at,
                deploy_source, actor_user_id, actor_username, actor_type, error_message,
                build_id, base_revision_id, change_summary
            FROM deployment_revisions
            WHERE guild_id = $1 AND status = 'success'
            ORDER BY deployed_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db_pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(PreviousSuccessfulRevision {
            revision_id: row.id,
            files: row.files.map(|files| files.0),
        }))
    }

    pub async fn get_deployment(&self, guild_id: &str) -> Result<Option<Deployment>> {
        if let Some(cached) = self.fetch_cached_deployment(guild_id).await? {
            return Ok(Some(cached));
        }

        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT guild_id, entry, files, source_map, bundle, created_at, updated_at
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
            SELECT guild_id, entry, files, source_map, bundle, created_at, updated_at
            FROM deployments
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut deployments = Vec::with_capacity(rows.len());
        for row in rows {
            deployments.push(to_deployment(row)?);
        }
        Ok(deployments)
    }

    pub fn summarize_changes(
        next_files: Option<&Vec<DeploymentFile>>,
        base_files: Option<&Vec<DeploymentFile>>,
    ) -> Option<DeploymentChangeSummary> {
        if next_files.is_none() && base_files.is_none() {
            return None;
        }

        let mut next_by_path = HashMap::new();
        if let Some(files) = next_files {
            for file in files {
                next_by_path.insert(file.path.clone(), file.contents.clone());
            }
        }

        let mut base_by_path = HashMap::new();
        if let Some(files) = base_files {
            for file in files {
                base_by_path.insert(file.path.clone(), file.contents.clone());
            }
        }

        let mut added_files = 0usize;
        let mut removed_files = 0usize;
        let mut modified_files = 0usize;

        for (path, next_contents) in &next_by_path {
            match base_by_path.get(path) {
                None => {
                    added_files += 1;
                }
                Some(base_contents) => {
                    if base_contents != next_contents {
                        modified_files += 1;
                    }
                }
            }
        }

        for path in base_by_path.keys() {
            if !next_by_path.contains_key(path) {
                removed_files += 1;
            }
        }

        Some(DeploymentChangeSummary {
            added_files,
            removed_files,
            modified_files,
        })
    }

    async fn cache_deployment(&self, deployment: &Deployment) -> Result<()> {
        let key = cache_key(&deployment.guild_id);
        let value = serde_json::to_string(deployment)?;
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
        files: row.files.map(|files| files.0),
        source_map: row.source_map.map(|source_map| source_map.0),
        bundle: row.bundle,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn to_revision(row: DeploymentRevisionRow) -> Result<DeploymentRevision> {
    Ok(DeploymentRevision {
        id: row.id,
        guild_id: row.guild_id,
        entry: row.entry,
        files: row.files.map(|files| files.0),
        bundle: row.bundle,
        source_map: row.source_map.map(|source_map| source_map.0),
        status: DeploymentRevisionStatus::from_str(&row.status)?,
        deployed_at: row.deployed_at,
        deploy_source: DeploymentSource::from_str(&row.deploy_source)?,
        actor_user_id: row.actor_user_id,
        actor_username: row.actor_username,
        actor_type: DeploymentActorType::from_str(&row.actor_type)?,
        error_message: row.error_message,
        build_id: row.build_id,
        base_revision_id: row.base_revision_id,
        change_summary: row.change_summary.map(|summary| summary.0),
    })
}

impl Deployment {
    pub fn module_name(&self) -> String {
        format!("guild:{}.bundle.js", self.guild_id)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{Deployment, DeploymentFile, DeploymentService, DeploymentSourceMapFile};

    #[test]
    fn deployment_json_roundtrip_preserves_source_map() {
        let deployment = Deployment {
            guild_id: "123".to_string(),
            entry: "src/main.ts".to_string(),
            files: Some(vec![DeploymentFile {
                path: "src/main.ts".to_string(),
                contents: "console.log('source')".to_string(),
            }]),
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
        assert!(deployment.files.is_none());
    }

    #[test]
    fn summarize_changes_counts_modified_paths() {
        let current = vec![DeploymentFile {
            path: "src/main.ts".to_string(),
            contents: "console.log('next')".to_string(),
        }];
        let base = vec![DeploymentFile {
            path: "src/main.ts".to_string(),
            contents: "console.log('base')".to_string(),
        }];

        let summary =
            DeploymentService::summarize_changes(Some(&current), Some(&base)).expect("summary");

        assert_eq!(summary.added_files, 0);
        assert_eq!(summary.removed_files, 0);
        assert_eq!(summary.modified_files, 1);
    }
}
