mod constants;
mod js;
mod limits;
mod secrets;
mod types;
mod worker;

#[cfg(test)]
mod tests;

use crate::{deployments::Deployment, kv::KvService, metrics::metrics, secrets::SecretService};
use constants::{DROPPABLE_EVENTS, MAX_DROPPABLE_BACKLOG, MAX_WORKERS_LIMIT};
use deno_core::error::AnyError;
use flora_config::RuntimeConfig;
use limits::RuntimeLimits;
use serde_json::Value;
use serenity::http::Http;
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};
use tokio::runtime::Builder;
use tracing::{error, info};
use types::{QueuedGuildEvent, Worker};
use worker::spawn_worker;

/// The main runtime that manages a pool of worker threads.
/// Each worker can host multiple guild isolates.
pub struct BotRuntime {
    workers: Vec<Worker>,
    num_workers: usize,
    secrets: Arc<SecretService>,
    guild_routes: Arc<parking_lot::Mutex<HashMap<String, usize>>>,
    migration_queues: Arc<parking_lot::Mutex<HashMap<String, Vec<QueuedGuildEvent>>>>,
}

impl BotRuntime {
    /// Create a new runtime with a pool of worker threads.
    pub fn new(
        http: Arc<Http>,
        kv: KvService,
        secrets: SecretService,
        config: RuntimeConfig,
    ) -> Self {
        let num_workers = config.max_workers.clamp(1, MAX_WORKERS_LIMIT);
        info!(target: "flora:runtime", num_workers, "spawning worker pool");
        let limits = RuntimeLimits::from_config(&config);

        let workers: Vec<Worker> = (0..num_workers)
            .map(|id| spawn_worker(id, http.clone(), kv.clone(), secrets.clone(), limits))
            .collect();

        Self {
            workers,
            num_workers,
            secrets: Arc::new(secrets),
            guild_routes: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            migration_queues: Arc::new(parking_lot::Mutex::new(HashMap::new())),
        }
    }

    /// Initialize all workers (creates default runtimes).
    /// Workers are initialized sequentially to avoid V8 race conditions.
    pub async fn initialize(&self) -> Result<(), AnyError> {
        for (i, worker) in self.workers.iter().enumerate() {
            worker.initialize().await?;
            info!(target: "flora:runtime", worker_id = i, "worker initialized");
        }
        info!(target: "flora:runtime", "all workers initialized");
        Ok(())
    }

    /// Load the SDK bundle into all workers' default runtimes.
    pub async fn load_sdk_bundle(&self, path: impl Into<PathBuf>) -> Result<(), AnyError> {
        let path = path.into();
        let futures: Vec<_> = self
            .workers
            .iter()
            .map(|w| w.load_sdk_bundle(path.clone()))
            .collect();
        futures::future::try_join_all(futures).await?;
        Ok(())
    }

    /// Load a user script into worker 0's default runtime (for local dev).
    #[allow(dead_code)]
    pub async fn load_local_script(&self, path: impl Into<PathBuf>) -> Result<(), AnyError> {
        let path = path.into();
        self.workers[0].load_user_script(path).await
    }

    /// Deploy a guild's script to the appropriate worker.
    pub async fn deploy_guild_script(&self, deployment: Deployment) -> Result<(), AnyError> {
        let worker_idx = {
            let mut routes = self.guild_routes.lock();
            match routes.get(&deployment.guild_id).copied() {
                Some(worker_idx) => worker_idx,
                None => {
                    let worker_idx = self.default_worker_for_guild(&deployment.guild_id);
                    routes.insert(deployment.guild_id.clone(), worker_idx);
                    worker_idx
                }
            }
        };
        info!(
            target: "flora:runtime",
            guild_id = deployment.guild_id,
            worker_idx,
            "routing guild deployment to worker"
        );
        self.workers[worker_idx].deploy_guild(deployment).await
    }

    #[allow(dead_code)]
    pub async fn migrate_guild_runtime(
        &self,
        guild_id: &str,
        target_worker: usize,
    ) -> Result<(), AnyError> {
        let target_worker = target_worker % self.num_workers;
        let source_worker = self.worker_for_guild(guild_id);
        if source_worker == target_worker {
            return Ok(());
        }

        self.begin_guild_migration(guild_id)?;

        let mut final_worker = source_worker;
        let mut migration_result = async {
            let envelope = self.workers[source_worker]
                .migrate_out(guild_id.to_string())
                .await?;

            match self.workers[target_worker]
                .migrate_in(guild_id.to_string(), envelope)
                .await
            {
                Ok(()) => {
                    self.guild_routes
                        .lock()
                        .insert(guild_id.to_string(), target_worker);
                    final_worker = target_worker;
                    metrics().migration_success();
                    info!(target: "flora:runtime", guild_id, source_worker, target_worker, "guild runtime migrated");
                    Ok(())
                }
                Err(failure) => {
                    let (err, envelope) = failure.into_parts();
                    if let Some(envelope) = envelope {
                        let rollback = self.workers[source_worker]
                            .migrate_in(guild_id.to_string(), envelope)
                            .await;
                        if let Err(rollback_failure) = rollback {
                            let (rollback_err, _) = rollback_failure.into_parts();
                            return Err(AnyError::msg(format!(
                                "migration failed and rollback failed: {}; rollback: {}",
                                err, rollback_err
                            )));
                        }
                    }
                    Err(err)
                }
            }
        }
        .await;

        let queued_events = self.finish_guild_migration(guild_id);
        let replay_result = self
            .flush_queued_events(guild_id, final_worker, queued_events)
            .await;
        if let Err(err) = replay_result {
            if migration_result.is_ok() {
                migration_result = Err(err);
            } else {
                error!(target: "flora:runtime", guild_id, ?err, "failed to replay queued events after migration error");
            }
        }

        migration_result
    }

    /// Dispatch a JS event to the appropriate runtime.
    pub async fn dispatch_js_event(
        &self,
        event: &str,
        guild_id: Option<String>,
        payload: Value,
    ) -> Result<(), AnyError> {
        match &guild_id {
            Some(gid) => {
                if self.enqueue_migrating_event(
                    gid,
                    QueuedGuildEvent {
                        event: event.to_string(),
                        payload: payload.clone(),
                    },
                ) {
                    return Ok(());
                }
                let worker_idx = self.worker_for_guild(gid);
                let worker = &self.workers[worker_idx];
                if is_droppable_event(event)
                    && worker.backlog.load(Ordering::Relaxed) >= MAX_DROPPABLE_BACKLOG
                {
                    info!(
                        target: "flora:runtime",
                        worker_id = worker.id,
                        guild_id = gid,
                        event,
                        backlog = worker.backlog.load(Ordering::Relaxed),
                        "dropping event due to backlog"
                    );
                    return Ok(());
                }
                self.workers[worker_idx]
                    .dispatch(guild_id, event.to_string(), payload)
                    .await
            }
            None => {
                let futures: Vec<_> = self
                    .workers
                    .iter()
                    .map(|w| w.broadcast(event.to_string(), payload.clone()))
                    .collect();
                futures::future::try_join_all(futures).await?;
                Ok(())
            }
        }
    }

    /// Refresh secrets for an existing guild runtime.
    pub async fn refresh_guild_secrets(&self, guild_id: &str) -> Result<(), AnyError> {
        let data = self
            .secrets
            .load_runtime(guild_id)
            .await
            .map_err(|err| AnyError::msg(err.to_string()))?;
        let worker_idx = self.worker_for_guild(guild_id);
        self.workers[worker_idx]
            .update_secrets(guild_id.to_string(), data)
            .await
    }

    fn worker_for_guild(&self, guild_id: &str) -> usize {
        let routes = self.guild_routes.lock();
        let Some(worker_idx) = routes.get(guild_id).copied() else {
            return self.default_worker_for_guild(guild_id);
        };
        worker_idx
    }

    fn default_worker_for_guild(&self, guild_id: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        guild_id.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_workers
    }

    fn begin_guild_migration(&self, guild_id: &str) -> Result<(), AnyError> {
        let mut queues = self.migration_queues.lock();
        if queues.contains_key(guild_id) {
            return Err(AnyError::msg("guild migration already in progress"));
        }
        queues.insert(guild_id.to_string(), Vec::new());
        Ok(())
    }

    fn finish_guild_migration(&self, guild_id: &str) -> Vec<QueuedGuildEvent> {
        self.migration_queues
            .lock()
            .remove(guild_id)
            .unwrap_or_default()
    }

    fn enqueue_migrating_event(&self, guild_id: &str, event: QueuedGuildEvent) -> bool {
        let mut queues = self.migration_queues.lock();
        let Some(queue) = queues.get_mut(guild_id) else {
            return false;
        };
        queue.push(event);
        true
    }

    async fn flush_queued_events(
        &self,
        guild_id: &str,
        worker_idx: usize,
        events: Vec<QueuedGuildEvent>,
    ) -> Result<(), AnyError> {
        let guild_id = guild_id.to_string();
        for QueuedGuildEvent { event, payload } in events {
            self.workers[worker_idx]
                .dispatch(Some(guild_id.clone()), event, payload)
                .await?;
        }
        Ok(())
    }

    async fn migrate_all_for_shutdown(&self) {
        if self.num_workers <= 1 {
            return;
        }

        let guild_ids: Vec<String> = {
            let routes = self.guild_routes.lock();
            routes.keys().cloned().collect()
        };
        for guild_id in guild_ids {
            let source_worker = self.worker_for_guild(&guild_id);
            let target_worker = (source_worker + 1) % self.num_workers;
            if source_worker == target_worker {
                continue;
            }
            if let Err(err) = self.migrate_guild_runtime(&guild_id, target_worker).await {
                error!(target: "flora:runtime", guild_id, source_worker, target_worker, ?err, "failed to migrate guild during shutdown");
            }
        }
    }
}

fn is_droppable_event(event: &str) -> bool {
    DROPPABLE_EVENTS.iter().any(|item| *item == event)
}

impl Drop for BotRuntime {
    fn drop(&mut self) {
        info!(target: "flora:runtime", "shutting down worker pool");
        if let Ok(rt) = Builder::new_current_thread().enable_all().build() {
            rt.block_on(self.migrate_all_for_shutdown());
        } else {
            error!(target: "flora:runtime", "failed to build runtime for graceful shutdown migration");
        }
        for worker in &self.workers {
            worker.send_shutdown();
        }
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
}
