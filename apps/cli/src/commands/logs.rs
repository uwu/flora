use crate::{api_types::LogEntry, auth::AuthRequestBuilder, config::CliConfig};
use color_eyre::eyre::Result;
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(crate) async fn logs(
    client: &Client,
    config: &CliConfig,
    guild: Option<String>,
    limit: usize,
) -> Result<()> {
    let url = match &guild {
        Some(guild_id) => format!("{}/logs/{}?limit={}", config.api_url, guild_id, limit),
        None => format!("{}/logs?limit={}", config.api_url, limit),
    };

    let logs: Vec<LogEntry> = client
        .get(&url)
        .maybe_bearer(&config.token)?
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if logs.is_empty() {
        println!("No logs found");
    } else {
        for entry in logs {
            print_log_entry(&entry);
        }
    }

    Ok(())
}

pub(crate) async fn stream_logs(
    client: &Client,
    config: &CliConfig,
    guild: Option<String>,
) -> Result<()> {
    let url = match &guild {
        Some(guild_id) => format!("{}/logs/{}/stream", config.api_url, guild_id),
        None => format!("{}/logs/stream", config.api_url),
    };

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    println!("Streaming logs... (press Ctrl+C to stop)");

    let response = client
        .get(&url)
        .maybe_bearer(&config.token)?
        .send()
        .await?
        .error_for_status()?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while running.load(Ordering::SeqCst) {
        let chunk = tokio::select! {
            chunk = stream.next() => chunk,
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                continue;
            }
        };

        match chunk {
            Some(Ok(bytes)) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(event_end) = buffer.find("\n\n") {
                    let event = buffer[..event_end].to_string();
                    buffer = buffer[event_end + 2..].to_string();

                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data: ")
                            && let Ok(entry) = serde_json::from_str::<LogEntry>(data)
                        {
                            print_log_entry(&entry);
                        }
                    }
                }
            }
            Some(Err(e)) => {
                eprintln!("Stream error: {}", e);
                break;
            }
            None => {
                println!("Stream ended");
                break;
            }
        }
    }

    println!("\nStopped streaming logs");
    Ok(())
}

pub(crate) fn print_log_entry(entry: &LogEntry) {
    use chrono::{TimeZone, Utc};

    let timestamp = Utc
        .timestamp_millis_opt(entry.timestamp)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| entry.timestamp.to_string());

    let level = match entry.level.as_str() {
        "error" => "\x1b[31mERROR\x1b[0m",
        "warn" => "\x1b[33mWARN\x1b[0m",
        "info" => "\x1b[32mINFO\x1b[0m",
        "debug" => "\x1b[34mDEBUG\x1b[0m",
        "trace" => "\x1b[90mTRACE\x1b[0m",
        other => other,
    };

    let guild = entry.guild_id.as_deref().unwrap_or("-");

    println!(
        "{} {} [{}] {}: {}",
        timestamp, level, guild, entry.target, entry.message
    );
}
