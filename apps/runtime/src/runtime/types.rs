use crate::{
    deployments::Deployment, metrics::metrics, ops::cron::CronJob, secrets::SecretsRuntimeData,
};
use deno_core::{
    JsRuntime,
    error::AnyError,
    v8::{self, Global},
};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};
use tokio::sync::{mpsc, oneshot};

/// Commands sent to worker threads.
pub(super) enum WorkerCommand {
    /// Initialize the worker's default runtime.
    Initialize {
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Load the SDK bundle into the default runtime.
    LoadSdkBundle {
        path: PathBuf,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Deploy a guild's script (creates/replaces guild isolate).
    DeployGuild {
        deployment: Deployment,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Dispatch an event to a specific guild's runtime.
    DispatchEvent {
        guild_id: Option<String>,
        event: String,
        payload: Value,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Broadcast an event to all runtimes on this worker.
    BroadcastEvent {
        event: String,
        payload: Value,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Refresh secrets for a guild runtime.
    UpdateSecrets {
        guild_id: String,
        secrets: Arc<SecretsRuntimeData>,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Move a guild runtime out of this worker.
    MigrateOut {
        guild_id: String,
        respond_to: oneshot::Sender<Result<MigrationEnvelope, AnyError>>,
    },
    /// Move a guild runtime into this worker.
    MigrateIn {
        guild_id: String,
        runtime: MigrationEnvelope,
        respond_to: oneshot::Sender<Result<(), MigrationInFailure>>,
    },
    /// Shutdown the worker.
    Shutdown,
}

/// A worker thread that owns multiple guild isolates.
pub(super) struct Worker {
    pub(super) id: usize,
    pub(super) sender: mpsc::UnboundedSender<WorkerCommand>,
    pub(super) handle: Option<thread::JoinHandle<()>>,
    pub(super) backlog: Arc<AtomicUsize>,
}

impl Worker {
    fn send_cmd(&self, cmd: WorkerCommand) -> Result<(), AnyError> {
        self.backlog.fetch_add(1, Ordering::Relaxed);
        if self.sender.send(cmd).is_err() {
            self.backlog.fetch_sub(1, Ordering::Relaxed);
            return Err(AnyError::msg("worker unavailable"));
        }
        Ok(())
    }

    pub(super) fn send_shutdown(&self) {
        self.backlog.fetch_add(1, Ordering::Relaxed);
        if self.sender.send(WorkerCommand::Shutdown).is_err() {
            self.backlog.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub(super) async fn initialize(&self) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::Initialize { respond_to: tx })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn load_sdk_bundle(&self, path: PathBuf) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::LoadSdkBundle {
            path,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn deploy_guild(&self, deployment: Deployment) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::DeployGuild {
            deployment,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn dispatch(
        &self,
        guild_id: Option<String>,
        event: String,
        payload: Value,
    ) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::DispatchEvent {
            guild_id,
            event,
            payload,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn broadcast(&self, event: String, payload: Value) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::BroadcastEvent {
            event,
            payload,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn update_secrets(
        &self,
        guild_id: String,
        secrets: Arc<SecretsRuntimeData>,
    ) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::UpdateSecrets {
            guild_id,
            secrets,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn migrate_out(
        &self,
        guild_id: String,
    ) -> Result<MigrationEnvelope, AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::MigrateOut {
            guild_id,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    pub(super) async fn migrate_in(
        &self,
        guild_id: String,
        runtime: MigrationEnvelope,
    ) -> Result<(), MigrationInFailure> {
        let (tx, rx) = oneshot::channel();
        self.backlog.fetch_add(1, Ordering::Relaxed);
        let cmd = WorkerCommand::MigrateIn {
            guild_id,
            runtime,
            respond_to: tx,
        };
        if let Err(err) = self.sender.send(cmd) {
            self.backlog.fetch_sub(1, Ordering::Relaxed);
            let WorkerCommand::MigrateIn { runtime, .. } = err.0 else {
                return Err(MigrationInFailure::new(
                    AnyError::msg("worker unavailable"),
                    None,
                ));
            };
            return Err(MigrationInFailure::new(
                AnyError::msg("worker unavailable"),
                Some(runtime),
            ));
        }
        match rx.await {
            Ok(result) => result,
            Err(_) => Err(MigrationInFailure::new(
                AnyError::msg("worker stopped"),
                None,
            )),
        }
    }
}

pub(super) struct JsRuntimeState {
    pub(super) runtime: JsRuntime,
    pub(super) dispatch_fn: Option<Global<v8::Function>>,
    pub(super) secrets: Arc<SecretsRuntimeData>,
}

pub(super) struct QueuedGuildEvent {
    pub(super) event: String,
    pub(super) payload: Value,
}

pub(super) struct MigrationEnvelope {
    runtime: Box<JsRuntimeState>,
    cron_jobs: Vec<CronJob>,
}

unsafe impl Send for MigrationEnvelope {}

impl MigrationEnvelope {
    pub(super) fn new(runtime: JsRuntimeState, cron_jobs: Vec<CronJob>) -> Self {
        Self {
            runtime: Box::new(runtime),
            cron_jobs,
        }
    }

    pub(super) fn into_parts(self) -> (JsRuntimeState, Vec<CronJob>) {
        (*self.runtime, self.cron_jobs)
    }
}

pub(super) struct MigrationInFailure {
    pub(super) error: AnyError,
    runtime: Option<MigrationEnvelope>,
}

impl MigrationInFailure {
    pub(super) fn new(error: AnyError, runtime: Option<MigrationEnvelope>) -> Self {
        Self { error, runtime }
    }

    pub(super) fn into_parts(self) -> (AnyError, Option<MigrationEnvelope>) {
        (self.error, self.runtime)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{stage} timed out")]
pub(super) struct RuntimeTimeout {
    pub(super) stage: &'static str,
}

impl Drop for JsRuntimeState {
    fn drop(&mut self) {
        metrics().isolate_destroyed();
        let dispatch_fn = self.dispatch_fn.take();
        if let Some(dispatch_fn) = dispatch_fn {
            let mut v8_guard = self.runtime.v8_guard();
            let _scope = v8::HandleScope::new(v8_guard.isolate());
            drop(dispatch_fn);
        }
    }
}

impl JsRuntimeState {
    pub(super) fn runtime(&self) -> &JsRuntime {
        &self.runtime
    }

    pub(super) fn runtime_mut(&mut self) -> &mut JsRuntime {
        &mut self.runtime
    }
}
