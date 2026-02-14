use deno_core::{FastStaticString, ascii_str_include};

pub(super) const MAX_WORKERS_LIMIT: usize = 64;
pub(super) const MAX_DROPPABLE_BACKLOG: usize = 2_000;
pub(super) const DROPPABLE_EVENTS: [&str; 2] = ["messageCreate", "messageUpdate"];
pub(super) const TERMINATION_GRACE_MS: u64 = 100;
pub(super) const RUNTIME_PRELUDE: &str =
    include_str!("../../../../runtime-dist/runtime_prelude.js");
pub(super) const SDK_BUNDLE_PATH: &str = "runtime-dist/runtime_sdk_bundle.js";
pub(super) const BOOTSTRAP_SPECIFIER: &str = "ext:flora_bootstrap/bootstrap.js";
pub(super) const BOOTSTRAP_DEPS: &[&str] = &[
    "deno_webidl",
    "deno_web",
    "deno_fetch",
    "deno_net",
    "deno_telemetry",
];
pub(super) const RUNTIME_BOOSTRAP: FastStaticString =
    ascii_str_include!("../../../../runtime-dist/runtime_bootstrap.js");
