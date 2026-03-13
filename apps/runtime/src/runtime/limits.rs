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
    pub(super) show_internal_stack_frames: bool,
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
            show_internal_stack_frames: config.show_internal_stack_frames,
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

#[cfg(test)]
mod tests {
    use flora_config::RuntimeConfig;

    use super::RuntimeLimits;

    #[test]
    fn from_config_copies_show_internal_stack_frames() {
        let config = RuntimeConfig {
            max_workers: 4,
            worker_queue_capacity: 32,
            boot_timeout_secs: 5,
            load_timeout_secs: 30,
            dispatch_timeout_secs: 3,
            rest_timeout_ms: 8_000,
            guild_concurrency: 4,
            max_script_bytes: 8_388_608,
            max_bundle_files: 200,
            max_bundle_total_bytes: 1_048_576,
            max_cron_jobs: 16,
            cron_timeout_secs: 5,
            migration_timeout_ms: 500,
            show_internal_stack_frames: true,
        };

        let limits = RuntimeLimits::from_config(&config);

        assert!(limits.show_internal_stack_frames);
    }
}
