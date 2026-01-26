use crate::ops::{CronRegistry, SharedCronRegistry};
use crate::{deployments::Deployment, kv::KvService, metrics::metrics, ops};
use deno_core::{
    Extension, ExtensionFileSource, FastStaticString, FastString, FsModuleLoader, JsRuntime,
    ModuleName, PollEventLoopOptions, RuntimeOptions, ascii_str_include,
    error::AnyError,
    serde_v8,
    v8::{self, Global},
};
use deno_permissions::{
    Permissions, PermissionsContainer, PermissionsOptions, RuntimePermissionDescriptorParser,
};
use flora_config::RuntimeConfig;
use serde_json::Value;
use serenity::http::Http;
use std::{
    borrow::Cow,
    collections::{HashMap, hash_map::DefaultHasher},
    future::Future,
    hash::{Hash, Hasher},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use sys_traits::impls::RealSys;
use tokio::{
    runtime::Builder,
    sync::{mpsc, oneshot},
    time::timeout,
};
use tracing::{error, info};

const MAX_WORKERS_LIMIT: usize = 64;
const MAX_DROPPABLE_BACKLOG: usize = 2_000;
const DROPPABLE_EVENTS: [&str; 2] = ["messageCreate", "messageUpdate"];
const TERMINATION_GRACE_MS: u64 = 100;
const RUNTIME_PRELUDE: &str = include_str!("../../../runtime-dist/runtime_prelude.js");
const SDK_BUNDLE_PATH: &str = "runtime-dist/runtime_sdk_bundle.js";
const BOOTSTRAP_SPECIFIER: &str = "ext:flora_bootstrap/bootstrap.js";
const BOOTSTRAP_DEPS: &[&str] = &[
    "deno_webidl",
    "deno_web",
    "deno_fetch",
    "deno_net",
    "deno_telemetry",
];
const RUNTIME_BOOSTRAP: FastStaticString =
    ascii_str_include!("../../../runtime-dist/runtime_bootstrap.js");

/// Commands sent to worker threads.
#[allow(dead_code)]
enum WorkerCommand {
    /// Initialize the worker's default runtime.
    Initialize {
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Load the SDK bundle into the default runtime.
    LoadSdkBundle {
        path: PathBuf,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    /// Load a user script into the default runtime.
    LoadUserScript {
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
    /// Unload a guild's runtime.
    UnloadGuild {
        guild_id: String,
        respond_to: oneshot::Sender<()>,
    },
    /// Shutdown the worker.
    Shutdown,
}

/// A worker thread that owns multiple guild isolates.
#[allow(dead_code)]
struct Worker {
    id: usize,
    sender: mpsc::UnboundedSender<WorkerCommand>,
    handle: Option<thread::JoinHandle<()>>,
    backlog: Arc<AtomicUsize>,
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

    fn send_shutdown(&self) {
        self.backlog.fetch_add(1, Ordering::Relaxed);
        if self.sender.send(WorkerCommand::Shutdown).is_err() {
            self.backlog.fetch_sub(1, Ordering::Relaxed);
        }
    }

    async fn initialize(&self) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::Initialize { respond_to: tx })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    async fn load_sdk_bundle(&self, path: PathBuf) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::LoadSdkBundle {
            path,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    #[allow(dead_code)]
    async fn load_user_script(&self, path: PathBuf) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::LoadUserScript {
            path,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    async fn deploy_guild(&self, deployment: Deployment) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::DeployGuild {
            deployment,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }

    async fn dispatch(
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

    async fn broadcast(&self, event: String, payload: Value) -> Result<(), AnyError> {
        let (tx, rx) = oneshot::channel();
        self.send_cmd(WorkerCommand::BroadcastEvent {
            event,
            payload,
            respond_to: tx,
        })?;
        rx.await.map_err(|_| AnyError::msg("worker stopped"))?
    }
}

/// The main runtime that manages a pool of worker threads.
/// Each worker can host multiple guild isolates.
pub struct BotRuntime {
    workers: Vec<Worker>,
    num_workers: usize,
}

#[derive(Clone, Copy)]
struct RuntimeLimits {
    boot_timeout: Option<Duration>,
    load_timeout: Option<Duration>,
    dispatch_timeout: Option<Duration>,
    cron_timeout: Option<Duration>,
    max_script_bytes: usize,
    max_cron_jobs: usize,
}

impl RuntimeLimits {
    fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            boot_timeout: timeout_from_secs(config.boot_timeout_secs),
            load_timeout: timeout_from_secs(config.load_timeout_secs),
            dispatch_timeout: timeout_from_secs(config.dispatch_timeout_secs),
            cron_timeout: timeout_from_secs(config.cron_timeout_secs),
            max_script_bytes: config.max_script_bytes,
            max_cron_jobs: config.max_cron_jobs,
        }
    }
}

fn timeout_from_secs(secs: u64) -> Option<Duration> {
    if secs == 0 {
        None
    } else {
        Some(Duration::from_secs(secs))
    }
}

impl BotRuntime {
    /// Create a new runtime with a pool of worker threads.
    pub fn new(http: Arc<Http>, kv: KvService, config: RuntimeConfig) -> Self {
        let num_workers = config.max_workers.clamp(1, MAX_WORKERS_LIMIT);
        info!(target: "flora:runtime", num_workers, "spawning worker pool");
        let limits = RuntimeLimits::from_config(&config);

        let workers: Vec<Worker> = (0..num_workers)
            .map(|id| spawn_worker(id, http.clone(), kv.clone(), limits))
            .collect();

        Self {
            workers,
            num_workers,
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
        let worker_idx = self.worker_for_guild(&deployment.guild_id);
        info!(
            target: "flora:runtime",
            guild_id = deployment.guild_id,
            worker_idx,
            "routing guild deployment to worker"
        );
        self.workers[worker_idx].deploy_guild(deployment).await
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
                // Broadcast to all workers (ready event, etc....)
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

    fn worker_for_guild(&self, guild_id: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        guild_id.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_workers
    }
}

fn is_droppable_event(event: &str) -> bool {
    DROPPABLE_EVENTS.iter().any(|item| *item == event)
}

impl Drop for BotRuntime {
    fn drop(&mut self) {
        info!(target: "flora:runtime", "shutting down worker pool");
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

fn spawn_worker(id: usize, http: Arc<Http>, kv: KvService, limits: RuntimeLimits) -> Worker {
    let (tx, rx) = mpsc::unbounded_channel();
    let backlog = Arc::new(AtomicUsize::new(0));
    let backlog_handle = Arc::clone(&backlog);

    let cron_registry = Arc::new(parking_lot::Mutex::new(CronRegistry::new(
        limits.max_cron_jobs,
    )));

    let handle = thread::Builder::new()
        .name(format!("flora-worker-{}", id))
        .spawn(move || {
            worker_thread(id, rx, http, kv, limits, backlog_handle, cron_registry);
        })
        .expect("failed to spawn worker thread");

    Worker {
        id,
        sender: tx,
        handle: Some(handle),
        backlog,
    }
}

fn worker_thread(
    worker_id: usize,
    mut receiver: mpsc::UnboundedReceiver<WorkerCommand>,
    http: Arc<Http>,
    kv: KvService,
    limits: RuntimeLimits,
    backlog: Arc<AtomicUsize>,
    cron_registry: SharedCronRegistry,
) {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build worker runtime");

    rt.block_on(async move {
        let mut guild_runtimes: HashMap<String, JsRuntimeState> = HashMap::new();
        let mut default_runtime: Option<JsRuntimeState> = None;
        let mut cron_interval = tokio::time::interval(Duration::from_secs(1));

        info!(target: "flora:runtime", worker_id, "worker thread started");

        loop {
            tokio::select! {
                _ = cron_interval.tick() => {
                    run_cron_tick(
                        &cron_registry,
                        &mut guild_runtimes,
                        &mut default_runtime,
                        worker_id,
                        &limits,
                    ).await;
                }
                cmd = receiver.recv() => {
                    let Some(cmd) = cmd else {
                        break;
                    };
                    backlog.fetch_sub(1, Ordering::Relaxed);
                    match cmd {
                        WorkerCommand::Initialize { respond_to } => {
                            let result =
                                initialize_worker_default(
                                    &mut default_runtime,
                                    &http,
                                    &kv,
                                    worker_id,
                                    &limits,
                                    cron_registry.clone(),
                                )
                                    .await;
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "failed to initialize worker");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::LoadSdkBundle { path, respond_to } => {
                            let result = match default_runtime.as_mut() {
                                Some(rt) => load_script_from_path(rt, path, worker_id, &limits).await,
                                None => Err(AnyError::msg("default runtime not initialized")),
                            };
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "failed to load SDK bundle");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::LoadUserScript { path, respond_to } => {
                            let result = match default_runtime.as_mut() {
                                Some(rt) => load_script_from_path(rt, path, worker_id, &limits).await,
                                None => Err(AnyError::msg("default runtime not initialized")),
                            };
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "failed to load user script");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::DeployGuild { deployment, respond_to } => {
                            {
                                let mut reg = cron_registry.lock();
                                reg.clear_guild(&deployment.guild_id);
                            }
                            let result = deploy_guild_to_worker(
                                &mut guild_runtimes,
                                &http,
                                &kv,
                                deployment,
                                worker_id,
                                &limits,
                                cron_registry.clone(),
                            )
                            .await;
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "failed to deploy guild");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::DispatchEvent { guild_id, event, payload, respond_to } => {
                            let result = dispatch_to_worker(
                                &mut guild_runtimes,
                                &mut default_runtime,
                                guild_id,
                                event,
                                payload,
                                worker_id,
                                &limits,
                            )
                            .await;
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "dispatch failed");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::BroadcastEvent { event, payload, respond_to } => {
                            let result = broadcast_to_worker(
                                &mut guild_runtimes,
                                &mut default_runtime,
                                event,
                                payload,
                                worker_id,
                                &limits,
                            )
                            .await;
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "broadcast failed");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::UnloadGuild { guild_id, respond_to } => {
                            {
                                let mut reg = cron_registry.lock();
                                reg.clear_guild(&guild_id);
                            }
                            guild_runtimes.remove(&guild_id);
                            info!(target: "flora:runtime", worker_id, guild_id, "unloaded guild");
                            let _ = respond_to.send(());
                        }

                        WorkerCommand::Shutdown => {
                            info!(target: "flora:runtime", worker_id, "worker shutting down");
                            break;
                        }
                    }
                }
            }
        }

        info!(target: "flora:runtime", worker_id, "worker thread exited");
    });
}

async fn run_cron_tick(
    cron_registry: &SharedCronRegistry,
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    default_runtime: &mut Option<JsRuntimeState>,
    worker_id: usize,
    limits: &RuntimeLimits,
) {
    use chrono::Utc;

    let now = Utc::now();
    let mut due_jobs = Vec::new();

    {
        let mut reg = cron_registry.lock();
        for (_gid, jobs) in reg.jobs.iter_mut() {
            for job in jobs.iter_mut() {
                if job.next_run <= now {
                    if job.skip_if_running && job.is_running {
                        info!(target: "flora:runtime", worker_id, name = job.name, "skipping cron job (still running)");
                        if let Ok(next) = job.schedule.find_next_occurrence(&now, false) {
                            job.next_run = next;
                        }
                        continue;
                    }
                    job.is_running = true;
                    due_jobs.push((
                        job.guild_id.clone(),
                        job.event_name.clone(),
                        job.name.clone(),
                    ));
                    if let Ok(next) = job.schedule.find_next_occurrence(&now, false) {
                        job.next_run = next;
                    }
                }
            }
        }
    }

    for (guild_id, event_name, cron_name) in due_jobs {
        let payload = serde_json::json!({
            "name": cron_name,
            "scheduledAt": now.to_rfc3339(),
        });

        let runtime = match &guild_id {
            Some(gid) => guild_runtimes.get_mut(gid),
            None => default_runtime.as_mut(),
        };

        let Some(runtime) = runtime else {
            mark_cron_not_running(cron_registry, &guild_id, &cron_name);
            continue;
        };

        let result =
            dispatch_cron_into_runtime(runtime, event_name.clone(), payload, worker_id, limits)
                .await;

        mark_cron_not_running(cron_registry, &guild_id, &cron_name);

        if let Err(ref err) = result {
            error!(target: "flora:runtime", worker_id, ?guild_id, cron_name, ?err, "cron dispatch failed");
        }
    }
}

fn mark_cron_not_running(
    cron_registry: &SharedCronRegistry,
    guild_id: &Option<String>,
    cron_name: &str,
) {
    let mut reg = cron_registry.lock();
    let Some(jobs) = reg.jobs.get_mut(guild_id) else {
        return;
    };
    for job in jobs.iter_mut() {
        if job.name == cron_name {
            job.is_running = false;
            break;
        }
    }
}

async fn dispatch_cron_into_runtime(
    js_state: &mut JsRuntimeState,
    event: String,
    payload: Value,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    let start = Instant::now();

    let Some(ref dispatch_fn) = js_state.dispatch_fn else {
        return Err(AnyError::msg("dispatch function not available"));
    };

    let context = js_state.runtime.main_context();
    let isolate = js_state.runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);

    let promise = {
        v8::scope_with_context!(scope, isolate, &context);
        let scope = scope;
        let context = v8::Local::new(scope, &context);
        let global = context.global(scope);

        let event_str = v8::String::new(scope, &event)
            .ok_or_else(|| AnyError::msg("Failed to create event string"))?;

        let payload_v8 = serde_v8::to_v8(scope, &payload)?;

        let dispatch_fn = v8::Local::new(scope, dispatch_fn);
        let args = [event_str.into(), payload_v8];
        let result = dispatch_fn
            .call(scope, global.into(), &args)
            .ok_or_else(|| AnyError::msg("Dispatch call failed"))?;

        let promise = v8::Local::<v8::Promise>::try_from(result)
            .map_err(|_| AnyError::msg("Dispatch did not return a promise"))?;
        Global::new(scope, promise)
    };

    let result = with_timeout(
        limits.cron_timeout,
        async {
            js_state
                .runtime
                .run_event_loop(PollEventLoopOptions::default())
                .await
                .map_err(AnyError::from)?;

            let context = js_state.runtime.main_context();
            v8::scope_with_context!(scope, js_state.runtime.v8_isolate(), &context);
            let scope = scope;
            let promise = v8::Local::new(scope, &promise);
            match promise.state() {
                v8::PromiseState::Rejected => {
                    let exception = promise.result(scope);
                    Err(AnyError::msg(exception.to_rust_string_lossy(scope)))
                }
                _ => Ok(()),
            }
        },
        "cron",
    )
    .await
    .map_err(|err| {
        error!(target: "flora:runtime", ?err, "Cron dispatch promise error");
        err
    });

    if let Err(ref err) = result {
        if err.is::<RuntimeTimeout>() {
            terminate_runtime(&mut js_state.runtime, worker_id, "cron").await;
        }
    }

    match &result {
        Ok(_) => metrics().dispatch_success(start.elapsed()),
        Err(_) => metrics().dispatch_error(),
    }

    result.map(|_| ())
}

struct JsRuntimeState {
    runtime: JsRuntime,
    dispatch_fn: Option<Global<v8::Function>>,
    #[allow(dead_code)]
    guild_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("{stage} timed out")]
struct RuntimeTimeout {
    stage: &'static str,
}

impl Drop for JsRuntimeState {
    fn drop(&mut self) {
        metrics().isolate_destroyed();
        // V8 requires the isolate to be entered before resetting persistent handles.
        if let Some(dispatch_fn) = self.dispatch_fn.take() {
            let isolate = self.runtime.v8_isolate();
            let _isolate_guard = IsolateEnterGuard::new(isolate);
            // Create a handle scope so V8 is happy when cleaning up persistent handles.
            let scope = v8::HandleScope::new(isolate);
            drop(dispatch_fn);
            // Explicitly drop the scope before leaving the isolate.
            drop(scope);
        }
    }
}

/// RAII guard that enters an isolate on construction and exits on drop.
/// This is required because V8's thread-local "current isolate" state
/// needs to be set correctly before any V8 operations. Sigh.
struct IsolateEnterGuard {
    isolate: *mut v8::OwnedIsolate,
}

impl IsolateEnterGuard {
    fn new(isolate: &mut v8::OwnedIsolate) -> Self {
        // Enter the isolate so subsequent scopes are tied to it.
        unsafe { isolate.enter() };
        Self { isolate }
    }
}

impl Drop for IsolateEnterGuard {
    fn drop(&mut self) {
        // SAFETY: isolate lives for the guard's lifetime, we only store the raw pointer.
        let isolate = unsafe { &mut *self.isolate };
        unsafe { isolate.exit() };
    }
}

fn enter_isolate(runtime: &mut JsRuntime) -> IsolateEnterGuard {
    let isolate = runtime.v8_isolate();
    IsolateEnterGuard::new(isolate)
}

async fn initialize_worker_default(
    default_runtime: &mut Option<JsRuntimeState>,
    http: &Arc<Http>,
    kv: &KvService,
    worker_id: usize,
    limits: &RuntimeLimits,
    cron_registry: SharedCronRegistry,
) -> Result<(), AnyError> {
    let mut runtime = new_js_runtime(http.clone(), kv.clone(), None, cron_registry);

    runtime
        .runtime
        .execute_script("flora:bootstrap", RUNTIME_PRELUDE)?;
    run_event_loop_with_timeout(
        &mut runtime.runtime,
        PollEventLoopOptions::default(),
        limits.boot_timeout,
        worker_id,
        "bootstrap",
    )
    .await?;

    // Extract dispatch function - need to enter isolate first
    let context = runtime.runtime.main_context();
    let isolate = runtime.runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);
    runtime.dispatch_fn = Some(extract_dispatch_fn_no_enter_impl(&context, isolate)?);

    info!(target: "flora:runtime", worker_id, "Default runtime initialized");
    *default_runtime = Some(runtime);
    Ok(())
}

async fn deploy_guild_to_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    http: &Arc<Http>,
    kv: &KvService,
    deployment: Deployment,
    worker_id: usize,
    limits: &RuntimeLimits,
    cron_registry: SharedCronRegistry,
) -> Result<(), AnyError> {
    let guild_id = deployment.guild_id.clone();
    let saved_runtime = guild_runtimes.remove(&guild_id);
    let saved_crons = {
        let reg = cron_registry.lock();
        reg.jobs.get(&Some(guild_id.clone())).cloned()
    };
    {
        let mut reg = cron_registry.lock();
        reg.clear_guild(&guild_id);
    }

    info!(target: "flora:runtime", worker_id, guild_id, "Creating guild runtime");

    let result = async {
        let mut runtime = new_js_runtime(
            http.clone(),
            kv.clone(),
            Some(guild_id.clone()),
            cron_registry.clone(),
        );

        {
            let _isolate_guard = enter_isolate(&mut runtime.runtime);
            let code = format!("globalThis.__floraGuildId = \"{}\";", guild_id);
            runtime
                .runtime
                .execute_script("flora:guild_context", code)?;
        }

        runtime
            .runtime
            .execute_script("flora:bootstrap", RUNTIME_PRELUDE)?;
        run_event_loop_with_timeout(
            &mut runtime.runtime,
            PollEventLoopOptions::default(),
            limits.boot_timeout,
            worker_id,
            "bootstrap",
        )
        .await?;

        {
            let context = runtime.runtime.main_context();
            let isolate = runtime.runtime.v8_isolate();
            let _isolate_guard = IsolateEnterGuard::new(isolate);
            runtime.dispatch_fn = Some(extract_dispatch_fn_no_enter_impl(&context, isolate)?);
        }

        info!(target: "flora:runtime", worker_id, guild_id, path = SDK_BUNDLE_PATH, "Loading SDK bundle");
        load_script_from_path(
            &mut runtime,
            PathBuf::from(SDK_BUNDLE_PATH),
            worker_id,
            limits,
        )
        .await?;

        let module_name = ModuleName::from(deployment.module_name());
        let script_name = module_name.as_str().to_string();
        info!(target: "flora:runtime", worker_id, guild_id, script = script_name, "Loading guild script");
        load_script_source(
            &mut runtime.runtime,
            module_name,
            deployment.bundle.clone(),
            script_name,
            worker_id,
            limits,
        )
        .await?;

        {
            let context = runtime.runtime.main_context();
            let isolate = runtime.runtime.v8_isolate();
            let _isolate_guard = IsolateEnterGuard::new(isolate);
            runtime.dispatch_fn = Some(extract_dispatch_fn_no_enter_impl(&context, isolate)?);
        }

        Ok::<JsRuntimeState, AnyError>(runtime)
    }
    .await;

    match result {
        Ok(runtime) => {
            if let Some(old) = guild_runtimes.insert(guild_id.clone(), runtime) {
                drop(old);
                info!(target: "flora:runtime", worker_id, guild_id, "Dropped old guild runtime");
            }
            info!(target: "flora:runtime", worker_id, guild_id, "Guild deployment loaded");
            Ok(())
        }
        Err(err) => {
            if let Some(old) = saved_runtime {
                guild_runtimes.insert(guild_id.clone(), old);
                info!(target: "flora:runtime", worker_id, guild_id, "Restored previous guild runtime after failed deploy");
            }
            if let Some(crons) = saved_crons {
                let mut reg = cron_registry.lock();
                reg.jobs.insert(Some(guild_id.clone()), crons);
            }
            Err(err)
        }
    }
}

async fn dispatch_to_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    default_runtime: &mut Option<JsRuntimeState>,
    guild_id: Option<String>,
    event: String,
    payload: Value,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    let runtime = match guild_id {
        Some(ref gid) => guild_runtimes
            .get_mut(gid)
            .ok_or_else(|| AnyError::msg("No runtime available for guild"))?,
        None => default_runtime
            .as_mut()
            .ok_or_else(|| AnyError::msg("No default runtime available"))?,
    };

    dispatch_into_runtime(runtime, event, payload, worker_id, limits).await
}

async fn broadcast_to_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    default_runtime: &mut Option<JsRuntimeState>,
    event: String,
    payload: Value,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    if let Some(runtime) = default_runtime {
        if let Err(err) =
            dispatch_into_runtime(runtime, event.clone(), payload.clone(), worker_id, limits).await
        {
            error!(target: "flora:runtime", worker_id, ?err, "Broadcast to default runtime failed");
        }
    }

    for (guild_id, runtime) in guild_runtimes.iter_mut() {
        if let Err(err) =
            dispatch_into_runtime(runtime, event.clone(), payload.clone(), worker_id, limits).await
        {
            error!(target: "flora:runtime", worker_id, guild_id, ?err, "Broadcast dispatch failed");
        }
    }

    Ok(())
}

async fn dispatch_into_runtime(
    js_state: &mut JsRuntimeState,
    event: String,
    payload: Value,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    let start = Instant::now();
    let dispatch_fn = js_state
        .dispatch_fn
        .as_ref()
        .ok_or_else(|| AnyError::msg("Dispatch function not initialized"))?;

    let context = js_state.runtime.main_context();
    let isolate = js_state.runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);

    let (event_value, payload_value) = {
        v8::scope_with_context!(scope, isolate, &context);
        let scope = scope;
        let event_value = serde_v8::to_v8(scope, &event)?;
        let payload_value = serde_v8::to_v8(scope, &payload)?;
        (
            v8::Global::new(scope, event_value),
            v8::Global::new(scope, payload_value),
        )
    };

    let call = js_state
        .runtime
        .call_with_args(dispatch_fn, &[event_value, payload_value]);
    let result = with_timeout(
        limits.dispatch_timeout,
        async {
            js_state
                .runtime
                .with_event_loop_promise(call, PollEventLoopOptions::default())
                .await
                .map_err(AnyError::from)
        },
        "dispatch",
    )
    .await
    .map_err(|err| {
        error!(target: "flora:runtime", ?err, "Dispatch promise error");
        err
    });

    if let Err(ref err) = result {
        if err.is::<RuntimeTimeout>() {
            terminate_runtime(&mut js_state.runtime, worker_id, "dispatch").await;
        }
    }

    match &result {
        Ok(_) => metrics().dispatch_success(start.elapsed()),
        Err(_) => metrics().dispatch_error(),
    }

    result.map(|_| ())
}

async fn load_script_from_path(
    js_state: &mut JsRuntimeState,
    path: PathBuf,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    let metadata = tokio::fs::metadata(&path).await?;
    let size = metadata.len() as usize;
    if size > limits.max_script_bytes {
        return Err(AnyError::msg(format!(
            "script exceeds size limit (max {} bytes)",
            limits.max_script_bytes
        )));
    }
    let source = tokio::fs::read_to_string(&path).await?;
    let name = path.to_string_lossy().to_string();

    load_script_source(
        &mut js_state.runtime,
        ModuleName::from(name.clone()),
        source,
        name,
        worker_id,
        limits,
    )
    .await
}

async fn load_script_source(
    js_runtime: &mut JsRuntime,
    module_name: ModuleName,
    source: String,
    name: String,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    if source.len() > limits.max_script_bytes {
        return Err(AnyError::msg(format!(
            "script exceeds size limit (max {} bytes)",
            limits.max_script_bytes
        )));
    }
    info!(target: "flora:runtime", worker_id, module = module_name.as_str(), "Executing module source");
    let _isolate_guard = enter_isolate(js_runtime);

    let code = match crate::transpile::transpile_if_typescript(&module_name, &source)? {
        Some(result) => result.code,
        None => FastString::from(source),
    };

    js_runtime.execute_script(name, code)?;
    run_event_loop_with_timeout(
        js_runtime,
        PollEventLoopOptions::default(),
        limits.load_timeout,
        worker_id,
        "module_load",
    )
    .await?;

    info!(target: "flora:runtime", worker_id, module = module_name.as_str(), "Module executed");
    Ok(())
}

async fn with_timeout<T>(
    timeout_duration: Option<Duration>,
    fut: impl Future<Output = Result<T, AnyError>>,
    stage: &'static str,
) -> Result<T, AnyError> {
    match timeout_duration {
        Some(duration) => match timeout(duration, fut).await {
            Ok(result) => result,
            Err(_) => Err(AnyError::from(RuntimeTimeout { stage })),
        },
        None => fut.await,
    }
}

async fn run_event_loop_with_timeout(
    runtime: &mut JsRuntime,
    poll_options: PollEventLoopOptions,
    timeout_duration: Option<Duration>,
    worker_id: usize,
    stage: &'static str,
) -> Result<(), AnyError> {
    let result = with_timeout(
        timeout_duration,
        async {
            runtime
                .run_event_loop(poll_options)
                .await
                .map_err(AnyError::from)
        },
        stage,
    )
    .await
    .map_err(|err| {
        error!(
            target: "flora:runtime",
            worker_id,
            stage,
            ?err,
            "event loop error"
        );
        err
    });

    if let Err(ref err) = result {
        if err.is::<RuntimeTimeout>() {
            terminate_runtime(runtime, worker_id, stage).await;
        }
    }

    result
}

async fn terminate_runtime(runtime: &mut JsRuntime, worker_id: usize, stage: &'static str) {
    let isolate = runtime.v8_isolate();
    let ok = isolate.terminate_execution();
    if !ok {
        error!(
            target: "flora:runtime",
            worker_id,
            stage,
            "failed to terminate execution"
        );
        return;
    }

    let _ = timeout(
        Duration::from_millis(TERMINATION_GRACE_MS),
        runtime.run_event_loop(PollEventLoopOptions::default()),
    )
    .await;

    let ok = runtime.v8_isolate().cancel_terminate_execution();
    if !ok {
        error!(
            target: "flora:runtime",
            worker_id,
            stage,
            "failed to cancel termination"
        );
    }
}

fn extract_dispatch_fn_no_enter_impl(
    context: &v8::Global<v8::Context>,
    isolate: &mut v8::OwnedIsolate,
) -> Result<Global<v8::Function>, AnyError> {
    v8::scope_with_context!(scope, isolate, context);
    let scope = scope;
    let context = v8::Local::new(scope, context);
    let global = context.global(scope);
    let key = v8::String::new(scope, "__floraDispatch")
        .ok_or_else(|| AnyError::msg("Failed to create dispatch name"))?;
    let value = global
        .get(scope, key.into())
        .ok_or_else(|| AnyError::msg("Dispatch function missing"))?;
    let function = v8::Local::<v8::Function>::try_from(value)
        .map_err(|_| AnyError::msg("Dispatch symbol is not a function"))?;
    Ok(Global::new(scope, function))
}

fn bootstrap_extension() -> Extension {
    Extension {
        name: "flora_bootstrap",
        deps: BOOTSTRAP_DEPS,
        esm_files: Cow::Owned(vec![ExtensionFileSource::new(
            BOOTSTRAP_SPECIFIER,
            RUNTIME_BOOSTRAP,
        )]),
        esm_entry_point: Some(BOOTSTRAP_SPECIFIER),
        ..Default::default()
    }
}

fn new_js_runtime(
    http: Arc<Http>,
    kv: KvService,
    guild_id: Option<String>,
    cron_registry: SharedCronRegistry,
) -> JsRuntimeState {
    metrics().isolate_created();
    let blob_store = Arc::new(deno_web::BlobStore::default());
    let broadcast_channel = deno_web::InMemoryBroadcastChannel::default();
    let descriptor_parser = Arc::new(RuntimePermissionDescriptorParser::new(RealSys));
    let permissions = PermissionsContainer::new(
        descriptor_parser.clone(),
        Permissions::from_options(
            descriptor_parser.as_ref(),
            &PermissionsOptions {
                allow_net: Some(vec![]),
                prompt: false,
                ..Default::default()
            },
        )
        .expect("failed to build runtime permissions"),
    );
    let runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![
            deno_telemetry::deno_telemetry::init(),
            deno_webidl::deno_webidl::init(),
            deno_web::deno_web::init(blob_store, None, broadcast_channel),
            deno_fetch::deno_fetch::init(deno_fetch::Options::default()),
            deno_net::deno_net::init(None, None),
            deno_tls::deno_tls::init(),
            bootstrap_extension(),
            ops::extension(http, kv.clone(), cron_registry),
        ],
        extension_transpiler: Some(Rc::new(|specifier, source| {
            match crate::transpile::transpile_if_typescript(&specifier, source.as_str())? {
                Some(result) => Ok((result.code, result.source_map)),
                None => Ok((source, None)),
            }
        })),
        module_loader: Some(Rc::new(FsModuleLoader)),
        ..Default::default()
    });
    runtime.op_state().borrow_mut().put(permissions);

    if let Some(ref gid) = guild_id {
        runtime.op_state().borrow_mut().put(gid.clone());
    }

    JsRuntimeState {
        runtime,
        dispatch_fn: None,
        guild_id,
    }
}
