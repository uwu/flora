use super::{
    constants::{BOOTSTRAP_DEPS, BOOTSTRAP_SPECIFIER, RUNTIME_BOOSTRAP, TERMINATION_GRACE_MS},
    limits::RuntimeLimits,
    secrets::secret_request_builder_hook,
    types::{JsRuntimeState, RuntimeTimeout},
};
use crate::{
    metrics::metrics,
    ops,
    ops::{SharedCronRegistry, interaction::CommandHashCache},
    services::{kv::KvService, secrets::SecretsRuntimeData},
};
use deno_core::{
    Extension, ExtensionFileSource, FastString, FsModuleLoader, JsRuntime, ModuleName,
    PollEventLoopOptions, RuntimeOptions,
    error::AnyError,
    v8::{self, Global},
};
use deno_permissions::{
    Permissions, PermissionsContainer, PermissionsOptions, RuntimePermissionDescriptorParser,
};
use serenity::http::Http;
use std::{borrow::Cow, future::Future, path::PathBuf, rc::Rc, sync::Arc, time::Duration};
use sys_traits::impls::RealSys;
use tokio::time::timeout;
use tracing::{error, info};

pub(super) async fn load_script_from_path(
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
        js_state.runtime_mut(),
        ModuleName::from(name.clone()),
        source,
        name,
        worker_id,
        limits,
    )
    .await
}

pub(super) async fn load_script_source(
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

pub(super) async fn with_timeout<T>(
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

pub(super) async fn run_event_loop_with_timeout(
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

pub(super) async fn terminate_runtime(
    runtime: &mut JsRuntime,
    worker_id: usize,
    stage: &'static str,
) {
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

pub(super) fn extract_dispatch_fn_no_enter_impl(
    context: &v8::Global<v8::Context>,
    isolate: &mut v8::Isolate,
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

pub(super) fn new_js_runtime(
    http: Arc<Http>,
    kv: KvService,
    secrets: Arc<SecretsRuntimeData>,
    guild_id: Option<String>,
    cron_registry: SharedCronRegistry,
) -> JsRuntimeState {
    metrics().isolate_created();
    let use_v8_locker = guild_id.is_some();
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
            deno_fetch::deno_fetch::init(deno_fetch::Options {
                request_builder_hook: Some(secret_request_builder_hook),
                ..Default::default()
            }),
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
        use_v8_locker,
        ..Default::default()
    });
    runtime.op_state().borrow_mut().put(permissions);
    runtime.op_state().borrow_mut().put(secrets.clone());
    runtime
        .op_state()
        .borrow_mut()
        .put(CommandHashCache::default());

    if let Some(ref gid) = guild_id {
        runtime.op_state().borrow_mut().put(gid.clone());
    }

    JsRuntimeState {
        runtime,
        dispatch_fn: None,
        secrets,
    }
}
