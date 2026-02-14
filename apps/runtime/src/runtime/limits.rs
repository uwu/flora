use flora_config::RuntimeConfig;
use std::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct RuntimeLimits {
    pub(super) boot_timeout: Option<Duration>,
    pub(super) load_timeout: Option<Duration>,
    pub(super) dispatch_timeout: Option<Duration>,
    pub(super) cron_timeout: Option<Duration>,
    pub(super) migration_timeout: Option<Duration>,
    pub(super) max_script_bytes: usize,
    pub(super) max_cron_jobs: usize,
}

impl RuntimeLimits {
    pub(super) fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            boot_timeout: timeout_from_secs(config.boot_timeout_secs),
            load_timeout: timeout_from_secs(config.load_timeout_secs),
            dispatch_timeout: timeout_from_secs(config.dispatch_timeout_secs),
            cron_timeout: timeout_from_secs(config.cron_timeout_secs),
            migration_timeout: timeout_from_millis(config.migration_timeout_ms),
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

fn timeout_from_millis(ms: u64) -> Option<Duration> {
    if ms == 0 {
        None
    } else {
        Some(Duration::from_millis(ms))
    }
}
