use std::{borrow::Cow, collections::HashMap, path::PathBuf, rc::Rc, sync::Arc, thread};

use deno_core::{
    Extension, ExtensionFileSource, FastString, FsModuleLoader, JsRuntime, ModuleName,
    PollEventLoopOptions, RuntimeOptions, ascii_str_include,
    error::AnyError,
    serde_v8,
    v8::{self, Global},
};
use deno_permissions::{PermissionsContainer, RuntimePermissionDescriptorParser};
use serde_json::Value;
use serenity::http::Http;
use sys_traits::impls::RealSys;
use tokio::{
    runtime::Builder,
    sync::{mpsc, oneshot},
};
use tracing::{error, info};

use crate::{deployments::Deployment, kv::KvService, ops};

pub struct BotRuntime {
    sender: mpsc::UnboundedSender<RuntimeCommand>,
}

struct JsRuntimeState {
    runtime: JsRuntime,
    dispatch_fn: Option<Global<v8::Function>>,
    guild_id: Option<String>,
}

const BOOTSTRAP_SPECIFIER: &str = "ext:flora_bootstrap/bootstrap.js";
const BOOTSTRAP_DEPS: &[&str] =
    &["deno_webidl", "deno_web", "deno_fetch", "deno_net", "deno_telemetry"];

fn bootstrap_extension() -> Extension {
    Extension {
        name: "flora_bootstrap",
        deps: BOOTSTRAP_DEPS,
        esm_files: Cow::Owned(vec![ExtensionFileSource::new(
            BOOTSTRAP_SPECIFIER,
            ascii_str_include!("../scripts/runtime_bootstrap.js"),
        )]),
        esm_entry_point: Some(BOOTSTRAP_SPECIFIER),
        ..Default::default()
    }
}

impl Drop for JsRuntimeState {
    fn drop(&mut self) {
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

enum RuntimeCommand {
    Initialize {
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    LoadScript {
        path: PathBuf,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    LoadGuildDeployment {
        deployment: Deployment,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
    Dispatch {
        event: String,
        guild_id: Option<String>,
        payload: Value,
        respond_to: oneshot::Sender<Result<(), AnyError>>,
    },
}

struct RuntimeThreadState {
    default_runtime: JsRuntimeState,
    guild_runtimes: HashMap<String, JsRuntimeState>,
    http: Arc<Http>,
    kv: KvService,
}

impl BotRuntime {
    pub fn new(http: Arc<Http>, kv: KvService) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        thread::spawn(move || runtime_thread(receiver, http, kv));
        Self { sender }
    }

    pub async fn initialize(&self) -> Result<(), AnyError> {
        self.request(|respond_to| RuntimeCommand::Initialize { respond_to }).await
    }

    pub async fn load_user_script(&self, path: impl Into<PathBuf>) -> Result<(), AnyError> {
        let path = path.into();
        self.request(|respond_to| RuntimeCommand::LoadScript { path, respond_to }).await
    }

    pub async fn deploy_guild_script(&self, deployment: Deployment) -> Result<(), AnyError> {
        self.request(|respond_to| RuntimeCommand::LoadGuildDeployment {
            deployment: deployment.clone(),
            respond_to,
        })
        .await
    }

    pub async fn dispatch_js_event(
        &self,
        event: &str,
        guild_id: Option<String>,
        payload: Value,
    ) -> Result<(), AnyError> {
        let event = event.to_string();
        self.request(|respond_to| RuntimeCommand::Dispatch { event, guild_id, payload, respond_to })
            .await
    }

    async fn request<F>(&self, f: F) -> Result<(), AnyError>
    where
        F: FnOnce(oneshot::Sender<Result<(), AnyError>>) -> RuntimeCommand,
    {
        let (tx, rx) = oneshot::channel();
        let command = f(tx);
        self.sender.send(command).map_err(|_| AnyError::msg("runtime thread is unavailable"))?;

        rx.await.map_err(|_| AnyError::msg("runtime thread stopped"))?
    }
}

fn runtime_thread(
    mut receiver: mpsc::UnboundedReceiver<RuntimeCommand>,
    http: Arc<Http>,
    kv: KvService,
) {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build single-thread runtime");

    runtime.block_on(async move {
        let mut state = RuntimeThreadState {
            default_runtime: new_js_runtime(http.clone(), kv.clone(), None),
            guild_runtimes: HashMap::new(),
            http,
            kv,
        };

        info!("flora JS runtime thread started");
        while let Some(command) = receiver.recv().await {
            match command {
                RuntimeCommand::Initialize { respond_to } => {
                    let result = initialize_runtime(&mut state.default_runtime).await;
                    if let Err(err) = &result {
                        error!("runtime init error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
                RuntimeCommand::LoadScript { path, respond_to } => {
                    let result = load_script_from_path(&mut state.default_runtime, path).await;
                    if let Err(err) = &result {
                        error!("script load error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
                RuntimeCommand::LoadGuildDeployment { deployment, respond_to } => {
                    let result = load_guild_deployment(&mut state, deployment).await;
                    if let Err(err) = &result {
                        error!("guild deployment load error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
                RuntimeCommand::Dispatch { event, guild_id, payload, respond_to } => {
                    let result = dispatch_event(&mut state, event, guild_id, payload).await;
                    if let Err(err) = &result {
                        error!("dispatch error: {:?}", err);
                    }
                    let _ = respond_to.send(result);
                }
            };
        }
    });
}

fn new_js_runtime(http: Arc<Http>, kv: KvService, guild_id: Option<String>) -> JsRuntimeState {
    let blob_store = Arc::new(deno_web::BlobStore::default());
    let broadcast_channel = deno_web::InMemoryBroadcastChannel::default();
    let permissions =
        PermissionsContainer::allow_all(Arc::new(RuntimePermissionDescriptorParser::new(RealSys)));
    let runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![
            deno_telemetry::deno_telemetry::init(),
            deno_webidl::deno_webidl::init(),
            deno_web::deno_web::init(blob_store, None, broadcast_channel),
            deno_fetch::deno_fetch::init(deno_fetch::Options::default()),
            deno_net::deno_net::init(None, None),
            deno_tls::deno_tls::init(),
            bootstrap_extension(),
            ops::extension(http, kv.clone()),
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

    JsRuntimeState { runtime, dispatch_fn: None, guild_id }
}

fn set_guild_context(js_state: &mut JsRuntimeState, guild_id: &str) -> Result<(), AnyError> {
    let _isolate_guard = enter_isolate(&mut js_state.runtime);
    let code = format!("globalThis.__floraGuildId = \"{}\";", guild_id);
    js_state.runtime.execute_script("flora:guild_context", code)?;
    Ok(())
}

async fn initialize_runtime(js_state: &mut JsRuntimeState) -> Result<(), AnyError> {
    js_state.runtime.execute_script("flora:bootstrap", RUNTIME_PRELUDE)?;
    js_state.runtime.run_event_loop(PollEventLoopOptions::default()).await?;
    let context = js_state.runtime.main_context();
    let isolate = js_state.runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);
    js_state.dispatch_fn = Some(extract_dispatch_fn_no_enter_impl(&context, isolate)?);
    info!("flora JS runtime initialized");
    Ok(())
}

async fn load_script_from_path(
    js_state: &mut JsRuntimeState,
    path: PathBuf,
) -> Result<(), AnyError> {
    let source = tokio::fs::read_to_string(&path).await?;
    let name = path.to_string_lossy().to_string();
    load_script_source(&mut js_state.runtime, ModuleName::from(name.clone()), source, name).await
}

async fn load_script_source(
    js_runtime: &mut JsRuntime,
    module_name: ModuleName,
    source: String,
    name: String,
) -> Result<(), AnyError> {
    info!(target: "flora:runtime", module = module_name.as_str(), "executing module source");
    let _isolate_guard = enter_isolate(js_runtime);
    let code = match crate::transpile::transpile_if_typescript(&module_name, &source)? {
        Some(result) => result.code,
        None => FastString::from(source),
    };

    js_runtime.execute_script(name, code)?;
    js_runtime.run_event_loop(PollEventLoopOptions::default()).await?;
    info!(target: "flora:runtime", module = module_name.as_str(), "module executed");
    Ok(())
}

async fn load_guild_deployment(
    state: &mut RuntimeThreadState,
    deployment: Deployment,
) -> Result<(), AnyError> {
    // Drop any existing isolate for this guild before spinning up a fresh one.
    if let Some(old_runtime) = state.guild_runtimes.remove(&deployment.guild_id) {
        drop(old_runtime);
    }

    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        "creating guild runtime"
    );
    let mut runtime =
        new_js_runtime(state.http.clone(), state.kv.clone(), Some(deployment.guild_id.clone()));
    set_guild_context(&mut runtime, &deployment.guild_id)?;
    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        "initializing guild runtime prelude"
    );
    initialize_runtime(&mut runtime).await?;
    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        path = SDK_BUNDLE_PATH,
        "loading sdk bundle"
    );
    load_script_from_path(&mut runtime, PathBuf::from(SDK_BUNDLE_PATH)).await?;

    let module_name = ModuleName::from(deployment.module_name());
    let script_name = module_name.as_str().to_string();
    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        script = script_name,
        "loading guild script source"
    );
    load_script_source(&mut runtime.runtime, module_name, deployment.bundle.clone(), script_name)
        .await?;

    // Ensure dispatch function is refreshed after loading user script.
    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        "extracting dispatch function"
    );
    runtime.dispatch_fn = Some(extract_dispatch_fn(&mut runtime.runtime)?);
    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        "dispatch function extracted"
    );
    state.guild_runtimes.insert(deployment.guild_id.clone(), runtime);
    info!(
        target: "flora:runtime",
        guild_id = deployment.guild_id,
        "loaded guild deployment into isolate"
    );
    Ok(())
}

async fn dispatch_event(
    state: &mut RuntimeThreadState,
    event: String,
    guild_id: Option<String>,
    payload: Value,
) -> Result<(), AnyError> {
    if let Some(guild_id) = guild_id {
        if let Some(runtime) = state.guild_runtimes.get_mut(&guild_id) {
            dispatch_into_runtime(runtime, event, payload).await
        } else {
            dispatch_into_runtime(&mut state.default_runtime, event, payload).await
        }
    } else {
        // Broadcast ready-style events to all runtimes, including the default one.
        let mut result =
            dispatch_into_runtime(&mut state.default_runtime, event.clone(), payload.clone()).await;

        for runtime in state.guild_runtimes.values_mut() {
            if let Err(err) = dispatch_into_runtime(runtime, event.clone(), payload.clone()).await {
                error!("dispatch error in guild runtime: {:?}", err);
                result = Err(err);
            }
        }

        result
    }
}

async fn dispatch_into_runtime(
    js_state: &mut JsRuntimeState,
    event: String,
    payload: Value,
) -> Result<(), AnyError> {
    let dispatch_fn = js_state
        .dispatch_fn
        .as_ref()
        .ok_or_else(|| AnyError::msg("dispatch function not initialized"))?;

    let context = js_state.runtime.main_context();
    let isolate = js_state.runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);
    {
        v8::scope_with_context!(scope, isolate, &context);
        let scope = scope;
        let context = v8::Local::new(scope, &context);
        let dispatch_fn = v8::Local::new(scope, dispatch_fn);
        let this = context.global(scope);
        let event_value = serde_v8::to_v8(scope, &event)?;
        let payload_value = serde_v8::to_v8(scope, &payload)?;

        dispatch_fn
            .call(scope, this.into(), &[event_value, payload_value])
            .ok_or_else(|| AnyError::msg("dispatch call failed"))?;
    }

    js_state.runtime.run_event_loop(PollEventLoopOptions::default()).await.map_err(AnyError::from)
}

fn extract_dispatch_fn(runtime: &mut JsRuntime) -> Result<Global<v8::Function>, AnyError> {
    let context = runtime.main_context();
    let isolate = runtime.v8_isolate();
    let _isolate_guard = IsolateEnterGuard::new(isolate);
    v8::scope_with_context!(scope, isolate, &context);
    let scope = scope;
    let context = v8::Local::new(scope, &context);
    let global = context.global(scope);
    let key = v8::String::new(scope, "__floraDispatch")
        .ok_or_else(|| AnyError::msg("failed to create dispatch name"))?;
    let value =
        global.get(scope, key.into()).ok_or_else(|| AnyError::msg("dispatch function missing"))?;
    let function = v8::Local::<v8::Function>::try_from(value)
        .map_err(|_| AnyError::msg("dispatch symbol is not a function"))?;
    Ok(Global::new(scope, function))
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
        .ok_or_else(|| AnyError::msg("failed to create dispatch name"))?;
    let value =
        global.get(scope, key.into()).ok_or_else(|| AnyError::msg("dispatch function missing"))?;
    let function = v8::Local::<v8::Function>::try_from(value)
        .map_err(|_| AnyError::msg("dispatch symbol is not a function"))?;
    Ok(Global::new(scope, function))
}

const RUNTIME_PRELUDE: &str = include_str!("../scripts/runtime_prelude.js");

const SDK_BUNDLE_PATH: &str = "dist/sdk-bundle.js";
