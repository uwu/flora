use dashmap::{DashMap, mapref::entry::Entry};
use rand::Rng;
use serenity::{Error as SerenityError, http::Http, model::id::GuildId};
use std::{
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::Semaphore,
    time::{sleep, timeout},
};
use tracing::{debug, info};

use crate::{ops::FloraError, services::scope_cache::ScopeCache};

pub struct DiscordRest {
    http: Arc<Http>,
    scope_cache: ScopeCache,
    guild_semaphores: DashMap<GuildId, Arc<Semaphore>>,
    config: RestConfig,
}

#[derive(Clone, Copy)]
pub(crate) struct RestConfig {
    pub max_wait: Duration,
    pub guild_concurrency: usize,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            max_wait: Duration::from_secs(8),
            guild_concurrency: 4,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum RestRetry {
    None,
    ReadOnly,
}

impl RestRetry {
    fn max_retries(self) -> usize {
        match self {
            Self::None => 0,
            Self::ReadOnly => 2,
        }
    }
}

impl DiscordRest {
    pub(crate) fn new(http: Arc<Http>, scope_cache: ScopeCache, config: RestConfig) -> Self {
        Self {
            http,
            scope_cache,
            guild_semaphores: DashMap::new(),
            config,
        }
    }

    pub(crate) fn scope_cache(&self) -> &ScopeCache {
        &self.scope_cache
    }

    pub(crate) fn http(&self) -> &Arc<Http> {
        &self.http
    }

    pub(crate) async fn execute<T, Fut, F>(
        &self,
        guild_id: GuildId,
        route: impl Into<String>,
        retry: RestRetry,
        request: F,
    ) -> Result<T, FloraError>
    where
        F: Fn(Arc<Http>) -> Fut,
        Fut: Future<Output = Result<T, SerenityError>>,
    {
        let route = route.into();
        let semaphore = self.semaphore_for_guild(guild_id);
        let available = semaphore.available_permits();
        if available == 0 {
            debug!(target: "flora:rest", event = "rejected", guild_id = guild_id.get(), route = route.as_str());
        } else {
            debug!(target: "flora:rest", event = "admitted", guild_id = guild_id.get(), route = route.as_str(), available);
        }

        let acquire_started = Instant::now();
        let permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| FloraError::discord_http(503, 0, "rest concurrency semaphore closed"))?;
        let wait_ms = acquire_started.elapsed().as_millis() as u64;
        debug!(target: "flora:rest", event = "acquired", guild_id = guild_id.get(), route = route.as_str(), wait_ms);
        let _permit = permit;

        let mut retries = 0usize;
        loop {
            let outcome = timeout(self.config.max_wait, request(self.http.clone())).await;
            let outcome = match outcome {
                Ok(result) => result,
                Err(_) => {
                    info!(target: "flora:rest", event = "timeout", guild_id = guild_id.get(), route = route.as_str());
                    return Err(FloraError::rate_limited(
                        self.config.max_wait.as_millis() as u64,
                        false,
                        route,
                    ));
                }
            };

            match outcome {
                Ok(value) => return Ok(value),
                Err(err) => {
                    if !should_retry(&err, retry) {
                        return Err(FloraError::from_serenity_error(err, Some(&route)));
                    }
                    if retries >= retry.max_retries() {
                        return Err(FloraError::from_serenity_error(err, Some(&route)));
                    }
                    let delay = backoff_delay(retries);
                    let attempt = retries + 1;
                    info!(
                        target: "flora:rest",
                        event = "retry",
                        guild_id = guild_id.get(),
                        route = route.as_str(),
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        ?err
                    );
                    retries += 1;
                    sleep(delay).await;
                }
            }
        }
    }

    pub(crate) fn semaphore_for_guild(&self, guild_id: GuildId) -> Arc<Semaphore> {
        match self.guild_semaphores.entry(guild_id) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => entry
                .insert(Arc::new(Semaphore::new(self.config.guild_concurrency)))
                .clone(),
        }
    }
}

fn should_retry(error: &SerenityError, retry: RestRetry) -> bool {
    match retry {
        RestRetry::None => false,
        RestRetry::ReadOnly => match error {
            SerenityError::Http(http_error) => match http_error {
                serenity::http::HttpError::UnsuccessfulRequest(response) => {
                    response.status_code.is_server_error()
                }
                serenity::http::HttpError::Request(_) => true,
                serenity::http::HttpError::RateLimitI64F64 => false,
                serenity::http::HttpError::RateLimitUtf8 => false,
                serenity::http::HttpError::InvalidWebhook => false,
                serenity::http::HttpError::InvalidHeader(_) => false,
                serenity::http::HttpError::ApplicationIdMissing => false,
                _ => false,
            },
            SerenityError::Io(_) => true,
            SerenityError::Json(_) => false,
            SerenityError::Model(_) => false,
            SerenityError::Token(_) => false,
            SerenityError::Url(_) => false,
            _ => false,
        },
    }
}

fn backoff_delay(retries: usize) -> Duration {
    let base_ms = 250u64;
    let cap_ms = 2_000u64;
    let scale = 1u64.checked_shl(retries as u32).unwrap_or(u64::MAX);
    let max_ms = base_ms.saturating_mul(scale).min(cap_ms);
    let jitter = rand::thread_rng().gen_range(0..=max_ms);
    Duration::from_millis(jitter)
}
