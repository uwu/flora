//! Runtime metrics for Flora.
//!
//! Provides Prometheus-style metrics for monitoring runtime health,
//! isolate lifecycle, and event dispatch performance.

use std::{
    sync::{
        RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

/// Global metrics instance.
static METRICS: Metrics = Metrics::new();

/// Runtime metrics collector.
pub struct Metrics {
    /// Total number of active guild isolates.
    isolate_count: AtomicU64,
    /// Total number of dispatched events.
    dispatch_total: AtomicU64,
    /// Total number of failed dispatches.
    dispatch_errors: AtomicU64,
    /// Total number of timeout errors.
    timeout_errors: AtomicU64,
    /// Total number of OOM errors.
    oom_errors: AtomicU64,
    /// Total number of isolate restarts due to crashes.
    isolate_restarts: AtomicU64,
    /// Total number of runtime thread restarts.
    runtime_restarts: AtomicU64,
    /// Dispatch latency samples (last 1000).
    dispatch_latencies: RwLock<LatencyTracker>,
}

/// Tracks latency samples for percentile calculations.
#[allow(dead_code)]
struct LatencyTracker {
    samples: Vec<u64>,
    index: usize,
    capacity: usize,
}

impl LatencyTracker {
    const fn new() -> Self {
        Self {
            samples: Vec::new(),
            index: 0,
            capacity: 1000,
        }
    }

    #[allow(dead_code)]
    fn record(&mut self, latency_us: u64) {
        if self.samples.len() < self.capacity {
            self.samples.push(latency_us);
        } else {
            self.samples[self.index] = latency_us;
        }
        self.index = (self.index + 1) % self.capacity;
    }

    fn percentile(&self, p: f64) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        // Use nearest-rank method: ceil((p/100) * n) - 1
        let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
        sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
    }

    fn average(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        self.samples.iter().sum::<u64>() / self.samples.len() as u64
    }
}

impl Metrics {
    const fn new() -> Self {
        Self {
            isolate_count: AtomicU64::new(0),
            dispatch_total: AtomicU64::new(0),
            dispatch_errors: AtomicU64::new(0),
            timeout_errors: AtomicU64::new(0),
            oom_errors: AtomicU64::new(0),
            isolate_restarts: AtomicU64::new(0),
            runtime_restarts: AtomicU64::new(0),
            dispatch_latencies: RwLock::new(LatencyTracker::new()),
        }
    }

    /// Increments the isolate count.
    #[allow(dead_code)]
    pub fn isolate_created(&self) {
        self.isolate_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the isolate count.
    #[allow(dead_code)]
    pub fn isolate_destroyed(&self) {
        self.isolate_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Records a successful dispatch with latency.
    #[allow(dead_code)]
    pub fn dispatch_success(&self, latency: std::time::Duration) {
        self.dispatch_total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut tracker) = self.dispatch_latencies.write() {
            tracker.record(latency.as_micros() as u64);
        }
    }

    /// Records a failed dispatch.
    #[allow(dead_code)]
    pub fn dispatch_error(&self) {
        self.dispatch_total.fetch_add(1, Ordering::Relaxed);
        self.dispatch_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a timeout error.
    #[allow(dead_code)]
    pub fn timeout_error(&self) {
        self.timeout_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Records an OOM error.
    #[allow(dead_code)]
    pub fn oom_error(&self) {
        self.oom_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Records an isolate restart.
    #[allow(dead_code)]
    pub fn isolate_restarted(&self) {
        self.isolate_restarts.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a runtime thread restart.
    #[allow(dead_code)]
    pub fn runtime_restarted(&self) {
        self.runtime_restarts.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns a snapshot of all metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        let (avg_latency_us, p50_latency_us, p95_latency_us, p99_latency_us) =
            if let Ok(tracker) = self.dispatch_latencies.read() {
                (
                    tracker.average(),
                    tracker.percentile(50.0),
                    tracker.percentile(95.0),
                    tracker.percentile(99.0),
                )
            } else {
                (0, 0, 0, 0)
            };

        MetricsSnapshot {
            isolate_count: self.isolate_count.load(Ordering::Relaxed),
            dispatch_total: self.dispatch_total.load(Ordering::Relaxed),
            dispatch_errors: self.dispatch_errors.load(Ordering::Relaxed),
            timeout_errors: self.timeout_errors.load(Ordering::Relaxed),
            oom_errors: self.oom_errors.load(Ordering::Relaxed),
            isolate_restarts: self.isolate_restarts.load(Ordering::Relaxed),
            runtime_restarts: self.runtime_restarts.load(Ordering::Relaxed),
            avg_latency_us,
            p50_latency_us,
            p95_latency_us,
            p99_latency_us,
        }
    }

    /// Formats metrics in Prometheus exposition format.
    pub fn prometheus_format(&self) -> String {
        let s = self.snapshot();
        format!(
            r#"# HELP flora_isolate_count Number of active guild isolates
# TYPE flora_isolate_count gauge
flora_isolate_count {}

# HELP flora_dispatch_total Total number of event dispatches
# TYPE flora_dispatch_total counter
flora_dispatch_total {}

# HELP flora_dispatch_errors_total Total number of failed dispatches
# TYPE flora_dispatch_errors_total counter
flora_dispatch_errors_total {}

# HELP flora_timeout_errors_total Total number of timeout errors
# TYPE flora_timeout_errors_total counter
flora_timeout_errors_total {}

# HELP flora_oom_errors_total Total number of OOM errors
# TYPE flora_oom_errors_total counter
flora_oom_errors_total {}

# HELP flora_isolate_restarts_total Total number of isolate restarts
# TYPE flora_isolate_restarts_total counter
flora_isolate_restarts_total {}

# HELP flora_runtime_restarts_total Total number of runtime thread restarts
# TYPE flora_runtime_restarts_total counter
flora_runtime_restarts_total {}

# HELP flora_dispatch_latency_avg_us Average dispatch latency in microseconds
# TYPE flora_dispatch_latency_avg_us gauge
flora_dispatch_latency_avg_us {}

# HELP flora_dispatch_latency_p50_us P50 dispatch latency in microseconds
# TYPE flora_dispatch_latency_p50_us gauge
flora_dispatch_latency_p50_us {}

# HELP flora_dispatch_latency_p95_us P95 dispatch latency in microseconds
# TYPE flora_dispatch_latency_p95_us gauge
flora_dispatch_latency_p95_us {}

# HELP flora_dispatch_latency_p99_us P99 dispatch latency in microseconds
# TYPE flora_dispatch_latency_p99_us gauge
flora_dispatch_latency_p99_us {}
"#,
            s.isolate_count,
            s.dispatch_total,
            s.dispatch_errors,
            s.timeout_errors,
            s.oom_errors,
            s.isolate_restarts,
            s.runtime_restarts,
            s.avg_latency_us,
            s.p50_latency_us,
            s.p95_latency_us,
            s.p99_latency_us,
        )
    }
}

/// Snapshot of metrics at a point in time.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct MetricsSnapshot {
    pub isolate_count: u64,
    pub dispatch_total: u64,
    pub dispatch_errors: u64,
    pub timeout_errors: u64,
    pub oom_errors: u64,
    pub isolate_restarts: u64,
    pub runtime_restarts: u64,
    pub avg_latency_us: u64,
    pub p50_latency_us: u64,
    pub p95_latency_us: u64,
    pub p99_latency_us: u64,
}

/// Returns the global metrics instance.
pub fn metrics() -> &'static Metrics {
    &METRICS
}

/// RAII guard for timing dispatch operations.
#[allow(dead_code)]
pub struct DispatchTimer {
    start: Instant,
}

#[allow(dead_code)]
impl DispatchTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn success(self) {
        metrics().dispatch_success(self.start.elapsed());
    }

    pub fn error(self) {
        metrics().dispatch_error();
    }
}

impl Default for DispatchTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_snapshot() {
        let m = Metrics::new();
        m.isolate_created();
        m.isolate_created();
        m.dispatch_success(std::time::Duration::from_micros(100));
        m.dispatch_error();
        m.timeout_error();

        let snap = m.snapshot();
        assert_eq!(snap.isolate_count, 2);
        assert_eq!(snap.dispatch_total, 2);
        assert_eq!(snap.dispatch_errors, 1);
        assert_eq!(snap.timeout_errors, 1);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut tracker = LatencyTracker::new();
        for i in 1..=100 {
            tracker.record(i);
        }

        assert_eq!(tracker.percentile(50.0), 50);
        assert_eq!(tracker.percentile(95.0), 95);
        assert_eq!(tracker.percentile(99.0), 99);
    }

    #[test]
    fn test_prometheus_format() {
        let m = Metrics::new();
        m.dispatch_success(std::time::Duration::from_micros(50));
        let output = m.prometheus_format();
        assert!(output.contains("flora_dispatch_total 1"));
        assert!(output.contains("# TYPE flora_isolate_count gauge"));
    }
}
