//! Log streaming infrastructure.
//!
//! Provides a log sink that captures runtime logs and broadcasts them
//! to connected SSE clients for real-time log tailing.

use std::sync::Arc;

use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::Level;

/// Maximum number of log entries to buffer.
const LOG_BUFFER_SIZE: usize = 1000;
/// Broadcast channel capacity for SSE subscribers.
const BROADCAST_CAPACITY: usize = 256;

/// Global log sink instance.
static LOG_SINK: std::sync::OnceLock<LogSink> = std::sync::OnceLock::new();

/// A log entry captured from the runtime.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LogEntry {
    /// Timestamp in milliseconds since Unix epoch.
    pub timestamp: i64,
    /// Log level (trace, debug, info, warn, error).
    pub level: String,
    /// Target/module that produced the log.
    pub target: String,
    /// Guild ID if applicable.
    pub guild_id: Option<String>,
    /// Log message.
    pub message: String,
}

/// Log sink that captures and broadcasts log entries.
pub struct LogSink {
    /// Broadcast sender for SSE subscribers.
    sender: broadcast::Sender<LogEntry>,
    /// Recent log buffer for new subscribers.
    buffer: Arc<RwLock<LogBuffer>>,
}

struct LogBuffer {
    entries: Vec<LogEntry>,
    index: usize,
}

impl LogBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            index: 0,
        }
    }

    fn push(&mut self, entry: LogEntry) {
        if self.entries.len() < self.entries.capacity() {
            self.entries.push(entry);
        } else {
            self.entries[self.index] = entry;
        }
        self.index = (self.index + 1) % self.entries.capacity();
    }

    fn recent(&self, count: usize) -> Vec<LogEntry> {
        let len = self.entries.len();
        if len == 0 {
            return Vec::new();
        }

        let count = count.min(len);
        let mut result = Vec::with_capacity(count);

        // If buffer is not full, entries are in order
        if len < self.entries.capacity() {
            let start = len.saturating_sub(count);
            result.extend(self.entries[start..].iter().cloned());
        } else {
            // Buffer is full, need to handle wrap-around
            // index points to the next write position (oldest entry when full)
            // We want the last `count` entries in chronological order
            // Start from (index + len - count) % len to get the oldest of the `count` entries
            let start = (self.index + len - count) % len;

            if start < self.index {
                // start...index-1 are older, but we also need index..end and 0..start to complete
                // Actually when start < index, the `count` entries span from start to index-1
                // Wait - that's only if count entries fit between start and index
                // For count=len (all entries), start=index, so we hit the else branch
                // For count<len, we have fewer entries, and they're contiguous from start
                result.extend(self.entries[start..self.index].iter().cloned());
            } else if start > self.index {
                // Wraps around: start..end, then 0..index
                result.extend(self.entries[start..].iter().cloned());
                result.extend(self.entries[..self.index].iter().cloned());
            } else {
                // start == index means we want all entries
                // Order: index..end, then 0..index
                result.extend(self.entries[self.index..].iter().cloned());
                result.extend(self.entries[..self.index].iter().cloned());
            }
        }

        result
    }

    fn filter_by_guild(&self, guild_id: &str, count: usize) -> Vec<LogEntry> {
        let len = self.entries.len();
        if len == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);

        // Iterate in reverse order (newest first)
        let iter: Box<dyn Iterator<Item = &LogEntry>> = if len < self.entries.capacity() {
            Box::new(self.entries.iter().rev())
        } else {
            // Full buffer - start from index-1 (newest) and go backwards
            let newest = if self.index == 0 {
                len - 1
            } else {
                self.index - 1
            };
            Box::new(
                self.entries[..=newest]
                    .iter()
                    .rev()
                    .chain(self.entries[newest + 1..].iter().rev()),
            )
        };

        for entry in iter {
            if result.len() >= count {
                break;
            }
            if entry.guild_id.as_deref() == Some(guild_id) {
                result.push(entry.clone());
            }
        }

        // Reverse to get chronological order
        result.reverse();
        result
    }
}

impl LogSink {
    /// Creates a new log sink.
    fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            sender,
            buffer: Arc::new(RwLock::new(LogBuffer::new(LOG_BUFFER_SIZE))),
        }
    }

    /// Records a log entry from the runtime.
    pub fn log(&self, level: Level, target: &str, guild_id: Option<String>, message: String) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: level.to_string().to_lowercase(),
            target: target.to_string(),
            guild_id,
            message,
        };

        // Add to buffer
        self.buffer.write().push(entry.clone());

        // Broadcast to subscribers (ignore if no subscribers)
        let _ = self.sender.send(entry);
    }

    /// Subscribe to log broadcasts.
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.sender.subscribe()
    }

    /// Get recent logs.
    pub fn recent(&self, count: usize) -> Vec<LogEntry> {
        self.buffer.read().recent(count)
    }

    /// Get recent logs for a specific guild.
    pub fn recent_for_guild(&self, guild_id: &str, count: usize) -> Vec<LogEntry> {
        self.buffer.read().filter_by_guild(guild_id, count)
    }
}

/// Returns the global log sink instance.
pub fn log_sink() -> &'static LogSink {
    LOG_SINK.get_or_init(LogSink::new)
}

/// Convenience function to log from the JS runtime.
pub fn log_js(level: Level, guild_id: Option<String>, message: String) {
    log_sink().log(level, "flora:js", guild_id, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_push_and_recent() {
        let mut buffer = LogBuffer::new(5);

        for i in 0..3 {
            buffer.push(LogEntry {
                timestamp: i,
                level: "info".to_string(),
                target: "test".to_string(),
                guild_id: None,
                message: format!("msg {}", i),
            });
        }

        let recent = buffer.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].timestamp, 1);
        assert_eq!(recent[1].timestamp, 2);
    }

    #[test]
    fn test_log_buffer_wrap_around() {
        let mut buffer = LogBuffer::new(3);

        for i in 0..5 {
            buffer.push(LogEntry {
                timestamp: i,
                level: "info".to_string(),
                target: "test".to_string(),
                guild_id: None,
                message: format!("msg {}", i),
            });
        }

        let recent = buffer.recent(3);
        assert_eq!(recent.len(), 3);
        // Should have entries 2, 3, 4 (oldest 0, 1 were overwritten)
        assert_eq!(recent[0].timestamp, 2);
        assert_eq!(recent[1].timestamp, 3);
        assert_eq!(recent[2].timestamp, 4);
    }

    #[test]
    fn test_log_buffer_filter_by_guild() {
        let mut buffer = LogBuffer::new(10);

        for i in 0..6 {
            buffer.push(LogEntry {
                timestamp: i,
                level: "info".to_string(),
                target: "test".to_string(),
                guild_id: if i % 2 == 0 {
                    Some("guild_a".to_string())
                } else {
                    Some("guild_b".to_string())
                },
                message: format!("msg {}", i),
            });
        }

        let guild_a = buffer.filter_by_guild("guild_a", 10);
        assert_eq!(guild_a.len(), 3);
        assert!(
            guild_a
                .iter()
                .all(|e| e.guild_id.as_deref() == Some("guild_a"))
        );
    }
}
