use super::{
    constants::{RUNTIME_PRELUDE, SDK_BUNDLE, SDK_BUNDLE_PATH},
    js::{
        extract_dispatch_fn_no_enter_impl, load_es_module_source, load_script_source,
        new_js_runtime, run_event_loop_with_timeout, terminate_runtime, with_timeout,
    },
    limits::RuntimeLimits,
    secrets::SecretScope,
    types::{
        JsRuntimeState, MigrationEnvelope, MigrationInFailure, RuntimeTimeout, Worker,
        WorkerCommand,
    },
};
use crate::{
    log_sink::log_sink,
    metrics::metrics,
    ops::{CronRegistry, SharedCronRegistry},
    services::{
        deployments::Deployment,
        discord_rest::DiscordRest,
        kv::KvService,
        secrets::{SecretService, SecretsRuntimeData},
    },
};
use deno_core::{
    ModuleSpecifier, PollEventLoopOptions,
    error::AnyError,
    serde_v8,
    v8::{self, Global},
};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tokio::{runtime::Builder, sync::mpsc};
use tracing::{error, info};

pub(super) fn spawn_worker(
    id: usize,
    http: Arc<DiscordRest>,
    kv: KvService,
    secrets: SecretService,
    limits: RuntimeLimits,
    queue_capacity: usize,
) -> Worker {
    let (tx, rx) = mpsc::channel(queue_capacity);
    let backlog = Arc::new(AtomicUsize::new(0));
    let backlog_handle = Arc::clone(&backlog);

    let cron_registry = Arc::new(parking_lot::Mutex::new(CronRegistry::new(
        limits.max_cron_jobs,
    )));

    let handle = thread::Builder::new()
        .name(format!("flora-worker-{}", id))
        .spawn(move || {
            worker_thread(
                id,
                rx,
                http,
                kv,
                secrets,
                limits,
                backlog_handle,
                cron_registry,
            );
        })
        .expect("failed to spawn worker thread");
    metrics().runtime_restarted();

    Worker {
        id,
        sender: tx,
        handle: Some(handle),
        backlog,
    }
}

#[allow(clippy::too_many_arguments)]
fn worker_thread(
    worker_id: usize,
    mut receiver: mpsc::Receiver<WorkerCommand>,
    http: Arc<DiscordRest>,
    kv: KvService,
    secrets: SecretService,
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
        let default_secrets = SecretsRuntimeData::empty();

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
                                    default_secrets.clone(),
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

                        WorkerCommand::LoadSdkBundle { respond_to } => {
                            let result = match default_runtime.as_mut() {
                                Some(rt) => {
                                    load_script_source(
                                        rt.runtime_mut(),
                                        SDK_BUNDLE_PATH.to_string().into(),
                                        SDK_BUNDLE.to_string(),
                                        SDK_BUNDLE_PATH.to_string(),
                                        worker_id,
                                        &limits,
                                    )
                                    .await
                                }
                                None => Err(AnyError::msg("default runtime not initialized")),
                            };
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, ?err, "failed to load SDK bundle");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::DeployGuild { deployment, respond_to } => {
                            let guild_id = deployment.guild_id.clone();
                            {
                                let mut reg = cron_registry.lock();
                                reg.clear_guild(&guild_id);
                            }
                            let result = deploy_guild_to_worker(
                                &mut guild_runtimes,
                                &http,
                                &kv,
                                &secrets,
                                deployment,
                                worker_id,
                                &limits,
                                cron_registry.clone(),
                            )
                            .await;
                            if let Err(ref err) = result {
                                emit_runtime_error(Some(guild_id.as_str()), "deploy", err, &limits);
                                error!(target: "flora:runtime", worker_id, guild_id, ?err, "failed to deploy guild");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::UndeployGuild { guild_id, respond_to } => {
                            let result = undeploy_guild_from_worker(
                                &mut guild_runtimes,
                                &guild_id,
                                &cron_registry,
                                worker_id,
                            );
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, guild_id, ?err, "failed to undeploy guild");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::DispatchEvent { guild_id, event, payload, respond_to } => {
                            let guild_id_for_log = guild_id.clone();
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
                                emit_runtime_error(guild_id_for_log.as_deref(), "dispatch", err, &limits);
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

                        WorkerCommand::UpdateSecrets { guild_id, secrets, respond_to } => {
                            let result = update_runtime_secrets(
                                &mut guild_runtimes,
                                &guild_id,
                                secrets,
                            );
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, guild_id, ?err, "failed to update secrets");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::MigrateOut { guild_id, respond_to } => {
                            let result = migrate_out_from_worker(
                                &mut guild_runtimes,
                                &guild_id,
                                worker_id,
                                &limits,
                                &cron_registry,
                            )
                            .await;
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, guild_id, ?err, "failed to migrate out guild runtime");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::MigrateIn { guild_id, runtime, respond_to } => {
                            let result = migrate_in_to_worker(
                                &mut guild_runtimes,
                                guild_id,
                                runtime,
                                &cron_registry,
                            );
                            if let Err(ref err) = result {
                                error!(target: "flora:runtime", worker_id, error = %err.error, "failed to migrate in guild runtime");
                            }
                            let _ = respond_to.send(result);
                        }

                        WorkerCommand::Shutdown => {
                            info!(target: "flora:runtime", worker_id, "worker shutting down");
                            break;
                        }
                    }
                }
            }
        }

        if let Some(runtime) = default_runtime.take() {
            drop_runtime_state(runtime);
        }
        for (_, runtime) in guild_runtimes.drain() {
            drop_runtime_state(runtime);
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
            emit_runtime_error(guild_id.as_deref(), "cron", err, limits);
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
    let _secret_scope = SecretScope::enter(js_state.secrets.clone());

    let dispatch_fn = js_state
        .dispatch_fn
        .as_ref()
        .ok_or_else(|| AnyError::msg("dispatch function not available"))?
        .clone();

    let context = js_state.runtime().main_context();
    let promise = {
        let mut v8_guard = js_state.runtime_mut().v8_guard();
        let isolate = v8_guard.isolate();
        v8::scope_with_context!(scope, isolate, &context);
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
                .runtime_mut()
                .run_event_loop(PollEventLoopOptions::default())
                .await
                .map_err(AnyError::from)?;

            let context = js_state.runtime().main_context();
            let mut v8_guard = js_state.runtime_mut().v8_guard();
            v8::scope_with_context!(scope, v8_guard.isolate(), &context);
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
            metrics().timeout_error();
            terminate_runtime(js_state.runtime_mut(), worker_id, "cron").await;
        } else if is_oom_error(err) {
            metrics().oom_error();
        }
    }

    match &result {
        Ok(_) => metrics().dispatch_success(start.elapsed()),
        Err(_) => metrics().dispatch_error(),
    }

    result.map(|_| ())
}

async fn initialize_worker_default(
    default_runtime: &mut Option<JsRuntimeState>,
    http: &Arc<DiscordRest>,
    kv: &KvService,
    secrets: Arc<SecretsRuntimeData>,
    worker_id: usize,
    limits: &RuntimeLimits,
    cron_registry: SharedCronRegistry,
) -> Result<(), AnyError> {
    let mut runtime = new_js_runtime(http.clone(), kv.clone(), secrets, None, cron_registry);

    runtime
        .runtime_mut()
        .execute_script("flora:bootstrap", RUNTIME_PRELUDE)?;
    run_event_loop_with_timeout(
        runtime.runtime_mut(),
        PollEventLoopOptions::default(),
        limits.boot_timeout,
        worker_id,
        "bootstrap",
    )
    .await?;

    let context = runtime.runtime().main_context();
    let mut v8_guard = runtime.runtime_mut().v8_guard();
    runtime.dispatch_fn = Some(extract_dispatch_fn_no_enter_impl(
        &context,
        v8_guard.isolate(),
    )?);

    info!(target: "flora:runtime", worker_id, "Default runtime initialized");
    *default_runtime = Some(runtime);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn deploy_guild_to_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    http: &Arc<DiscordRest>,
    kv: &KvService,
    secrets: &SecretService,
    deployment: Deployment,
    worker_id: usize,
    limits: &RuntimeLimits,
    cron_registry: SharedCronRegistry,
) -> Result<(), AnyError> {
    let guild_id = deployment.guild_id.clone();
    let mut saved_runtime = guild_runtimes.remove(&guild_id);
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
        let secrets_data = load_runtime_secrets(secrets, &guild_id).await?;
        let mut runtime = new_js_runtime(
            http.clone(),
            kv.clone(),
            secrets_data,
            Some(guild_id.clone()),
            cron_registry.clone(),
        );

        {
            let code = format!("globalThis.__floraGuildId = \"{}\";", guild_id);
            runtime
                .runtime_mut()
                .execute_script("flora:guild_context", code)?;
        }

        runtime
            .runtime_mut()
            .execute_script("flora:bootstrap", RUNTIME_PRELUDE)?;
        run_event_loop_with_timeout(
            runtime.runtime_mut(),
            PollEventLoopOptions::default(),
            limits.boot_timeout,
            worker_id,
            "bootstrap",
        )
        .await?;

        {
            let context = runtime.runtime().main_context();
            let mut v8_guard = runtime.runtime_mut().v8_guard();
            runtime.dispatch_fn =
                Some(extract_dispatch_fn_no_enter_impl(&context, v8_guard.isolate())?);
        }

        info!(target: "flora:runtime", worker_id, guild_id, path = SDK_BUNDLE_PATH, "Loading SDK bundle");
        load_script_source(
            runtime.runtime_mut(),
            SDK_BUNDLE_PATH.to_string().into(),
            SDK_BUNDLE.to_string(),
            SDK_BUNDLE_PATH.to_string(),
            worker_id,
            limits,
        )
        .await?;

        let module_specifier = ModuleSpecifier::parse(&deployment.module_specifier())
            .map_err(|err| AnyError::msg(format!("invalid deployment module specifier: {err}")))?;
        let script_name = module_specifier.as_str().to_string();
        info!(target: "flora:runtime", worker_id, guild_id, script = script_name, "Loading guild script");
        load_es_module_source(
            runtime.runtime_mut(),
            module_specifier,
            deployment.bundle.clone(),
            deployment.source_map.as_ref().map(|source_map| source_map.contents.as_str()),
            worker_id,
            limits,
        )
        .await?;

        {
            let context = runtime.runtime().main_context();
            let mut v8_guard = runtime.runtime_mut().v8_guard();
            runtime.dispatch_fn =
                Some(extract_dispatch_fn_no_enter_impl(&context, v8_guard.isolate())?);
        }

        Ok::<JsRuntimeState, AnyError>(runtime)
    }
    .await;

    match result {
        Ok(runtime) => {
            let mut restarted = saved_runtime.is_some();
            if let Some(old) = saved_runtime.take() {
                drop_runtime_state(old);
            }
            if let Some(old) = guild_runtimes.insert(guild_id.clone(), runtime) {
                restarted = true;
                drop_runtime_state(old);
            }
            if restarted {
                metrics().isolate_restarted();
            }
            info!(target: "flora:runtime", worker_id, guild_id, "Guild deployment loaded");
            Ok(())
        }
        Err(err) => {
            if let Some(old) = saved_runtime.take() {
                guild_runtimes.insert(guild_id.clone(), old);
                info!(target: "flora:runtime", worker_id, guild_id, "Restored previous guild runtime after failed deploy");
            }
            if let Some(crons) = saved_crons {
                let mut reg = cron_registry.lock();
                reg.jobs.insert(Some(guild_id.clone()), crons);
            }
            let message = user_visible_error_message(&err, limits.show_internal_stack_frames);
            Err(AnyError::msg(message))
        }
    }
}

fn undeploy_guild_from_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    guild_id: &str,
    cron_registry: &SharedCronRegistry,
    worker_id: usize,
) -> Result<(), AnyError> {
    if let Some(runtime) = guild_runtimes.remove(guild_id) {
        drop_runtime_state(runtime);
    }

    {
        let mut reg = cron_registry.lock();
        reg.clear_guild(guild_id);
    }

    info!(target: "flora:runtime", worker_id, guild_id, "Guild runtime undeployed");
    Ok(())
}

async fn load_runtime_secrets(
    secrets: &SecretService,
    guild_id: &str,
) -> Result<Arc<SecretsRuntimeData>, AnyError> {
    #[cfg(test)]
    let result = secrets.load_runtime_for_tests(guild_id).await;
    #[cfg(not(test))]
    let result = secrets.load_runtime(guild_id).await;

    result.map_err(|err| AnyError::msg(err.to_string()))
}

pub(super) fn drop_runtime_state(mut runtime: JsRuntimeState) {
    let isolate = runtime.runtime_mut().v8_isolate();
    if !isolate.is_current() {
        unsafe { isolate.enter() };
    }
    drop(runtime);
}

async fn migrate_out_from_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    guild_id: &str,
    worker_id: usize,
    limits: &RuntimeLimits,
    cron_registry: &SharedCronRegistry,
) -> Result<MigrationEnvelope, AnyError> {
    let Some(mut runtime) = guild_runtimes.remove(guild_id) else {
        return Err(AnyError::msg("guild runtime not loaded on worker"));
    };

    let mut cron_jobs = {
        let mut reg = cron_registry.lock();
        reg.jobs
            .remove(&Some(guild_id.to_string()))
            .unwrap_or_default()
    };
    for job in cron_jobs.iter_mut() {
        job.is_running = false;
    }

    let quiesce_started = Instant::now();
    let quiesce_result = with_timeout(
        limits.migration_timeout,
        async {
            runtime
                .runtime_mut()
                .run_event_loop(PollEventLoopOptions::default())
                .await
                .map_err(AnyError::from)
        },
        "migration_quiesce",
    )
    .await;
    if let Err(err) = quiesce_result {
        if err.is::<RuntimeTimeout>() {
            metrics().migration_timeout();
        }
        if !cron_jobs.is_empty() {
            let mut reg = cron_registry.lock();
            reg.jobs.insert(Some(guild_id.to_string()), cron_jobs);
        }
        guild_runtimes.insert(guild_id.to_string(), runtime);
        return Err(err);
    }
    metrics().migration_quiesce_duration(quiesce_started.elapsed());

    if !runtime.runtime_mut().is_idle_for_migration() {
        if !cron_jobs.is_empty() {
            let mut reg = cron_registry.lock();
            reg.jobs.insert(Some(guild_id.to_string()), cron_jobs);
        }
        guild_runtimes.insert(guild_id.to_string(), runtime);
        return Err(AnyError::msg("guild runtime is not idle for migration"));
    }

    #[cfg(debug_assertions)]
    {
        let isolate = runtime.runtime_mut().v8_isolate();
        debug_assert!(
            !isolate.is_current(),
            "guild isolate still entered during migration"
        );
        debug_assert!(
            !v8::Locker::is_locked(isolate),
            "guild isolate locker still held during migration"
        );
    }

    info!(target: "flora:runtime", worker_id, guild_id, "migrated guild runtime out");
    Ok(MigrationEnvelope::new(runtime, cron_jobs))
}

fn migrate_in_to_worker(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    guild_id: String,
    envelope: MigrationEnvelope,
    cron_registry: &SharedCronRegistry,
) -> Result<(), MigrationInFailure> {
    let (mut runtime, cron_jobs) = envelope.into_parts();

    let context = runtime.runtime().main_context();
    {
        let mut v8_guard = runtime.runtime_mut().v8_guard();
        v8::scope_with_context!(scope, v8_guard.isolate(), &context);
        let _ = scope;
    }

    if let Some(old) = guild_runtimes.insert(guild_id.clone(), runtime) {
        drop_runtime_state(old);
    }
    if !cron_jobs.is_empty() {
        let mut reg = cron_registry.lock();
        reg.jobs.insert(Some(guild_id.clone()), cron_jobs);
    }

    info!(target: "flora:runtime", guild_id, "migrated guild runtime in");
    Ok(())
}

fn update_runtime_secrets(
    guild_runtimes: &mut HashMap<String, JsRuntimeState>,
    guild_id: &str,
    secrets: Arc<SecretsRuntimeData>,
) -> Result<(), AnyError> {
    let Some(runtime) = guild_runtimes.get_mut(guild_id) else {
        return Ok(());
    };
    runtime.secrets = secrets.clone();
    runtime.runtime_mut().op_state().borrow_mut().put(secrets);
    Ok(())
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
    if let Some(runtime) = default_runtime
        && let Err(err) =
            dispatch_into_runtime(runtime, event.clone(), payload.clone(), worker_id, limits).await
    {
        emit_runtime_error(None, "broadcast", &err, limits);
        error!(target: "flora:runtime", worker_id, ?err, "Broadcast to default runtime failed");
    }

    for (guild_id, runtime) in guild_runtimes.iter_mut() {
        if let Err(err) =
            dispatch_into_runtime(runtime, event.clone(), payload.clone(), worker_id, limits).await
        {
            emit_runtime_error(Some(guild_id.as_str()), "broadcast", &err, limits);
            error!(target: "flora:runtime", worker_id, guild_id, ?err, "Broadcast dispatch failed");
        }
    }

    Ok(())
}

pub(super) async fn dispatch_into_runtime(
    js_state: &mut JsRuntimeState,
    event: String,
    payload: Value,
    worker_id: usize,
    limits: &RuntimeLimits,
) -> Result<(), AnyError> {
    let start = Instant::now();
    let _secret_scope = SecretScope::enter(js_state.secrets.clone());
    let dispatch_fn = js_state
        .dispatch_fn
        .as_ref()
        .ok_or_else(|| AnyError::msg("Dispatch function not initialized"))?
        .clone();

    let context = js_state.runtime().main_context();
    let (event_value, payload_value) = {
        let mut v8_guard = js_state.runtime_mut().v8_guard();
        let isolate = v8_guard.isolate();
        v8::scope_with_context!(scope, isolate, &context);
        let event_value = serde_v8::to_v8(scope, &event)?;
        let payload_value = serde_v8::to_v8(scope, &payload)?;
        (
            v8::Global::new(scope, event_value),
            v8::Global::new(scope, payload_value),
        )
    };

    let call = js_state
        .runtime_mut()
        .call_with_args(&dispatch_fn, &[event_value, payload_value]);
    let result = with_timeout(
        limits.dispatch_timeout,
        async {
            js_state
                .runtime_mut()
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
            metrics().timeout_error();
            terminate_runtime(js_state.runtime_mut(), worker_id, "dispatch").await;
        } else if is_oom_error(err) {
            metrics().oom_error();
        }
    }

    match &result {
        Ok(_) => metrics().dispatch_success(start.elapsed()),
        Err(_) => metrics().dispatch_error(),
    }

    result.map(|_| ())
}

fn is_oom_error(err: &AnyError) -> bool {
    err.to_string()
        .to_ascii_lowercase()
        .contains("out of memory")
}

fn emit_runtime_error(guild_id: Option<&str>, stage: &str, err: &AnyError, limits: &RuntimeLimits) {
    let message = user_visible_error_message(err, limits.show_internal_stack_frames);
    log_sink().log(
        tracing::Level::ERROR,
        "flora:runtime",
        guild_id.map(ToOwned::to_owned),
        format!("{stage} failed: {message}"),
    );
}

fn user_visible_error_message(err: &AnyError, show_internal_stack_frames: bool) -> String {
    let text = err.to_string();
    if show_internal_stack_frames {
        return text;
    }

    let mut hidden_frames = 0usize;
    let mut lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index == 0 {
            lines.push(line.to_string());
            continue;
        }

        if is_internal_stack_frame(line) {
            hidden_frames += 1;
            continue;
        }

        lines.push(line.to_string());
    }

    if hidden_frames > 0 {
        lines.push(format!(
            "    ... {hidden_frames} internal frame(s) hidden (set RUNTIME_SHOW_INTERNAL_STACK_FRAMES=true to show)"
        ));
    }

    lines.join("\n")
}

fn is_internal_stack_frame(line: &str) -> bool {
    let line = line.trim().to_ascii_lowercase();
    if line.is_empty() {
        return false;
    }

    [
        "runtime-dist/",
        "runtime_sdk_bundle.js",
        "runtime_prelude.js",
        "runtime_bootstrap.js",
        "runtime_module_resolution.js",
        "flora:bootstrap",
        "flora:guild_context",
        "ext:",
    ]
    .iter()
    .any(|pattern| line.contains(pattern))
}

#[cfg(test)]
mod tests {
    use deno_core::error::AnyError;

    use super::{is_internal_stack_frame, user_visible_error_message};

    #[test]
    fn user_visible_error_message_hides_internal_frames_by_default() {
        let raw = "Error: boom\n    at handler (file:///guild/123/src/main.ts:4:3)\n    at op_dispatch (ext:core/ops.js:12:3)\n    at bootstrap (file:///runtime-dist/runtime_bootstrap.js:8:9)";
        let err = AnyError::msg(raw);

        let message = user_visible_error_message(&err, false);

        assert!(message.contains("Error: boom"));
        assert!(message.contains("file:///guild/123/src/main.ts"));
        assert!(!message.contains("ext:core/ops.js"));
        assert!(!message.contains("runtime-dist/runtime_bootstrap.js"));
        assert!(message.contains("2 internal frame(s) hidden"));
    }

    #[test]
    fn user_visible_error_message_keeps_internal_frames_when_enabled() {
        let raw = "Error: boom\n    at op_dispatch (ext:core/ops.js:12:3)";
        let err = AnyError::msg(raw);

        assert_eq!(user_visible_error_message(&err, true), raw);
    }

    #[test]
    fn is_internal_stack_frame_matches_runtime_and_ext_frames() {
        assert!(is_internal_stack_frame(
            "    at op_dispatch (ext:core/ops.js:12:3)"
        ));
        assert!(is_internal_stack_frame(
            "    at bootstrap (file:///runtime-dist/runtime_prelude.js:1:1)",
        ));
        assert!(!is_internal_stack_frame(
            "    at handler (file:///guild/123/src/main.ts:4:3)",
        ));
    }
}
