use dashmap::DashMap;
use fred::{prelude::*, types::Expiration};
use serenity::{
    http::Http,
    model::{
        channel::Channel,
        id::{ChannelId, GuildId, ThreadId, WebhookId},
    },
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{info, warn};

use crate::ops::FloraError;

const L1_TTL: Duration = Duration::from_secs(300);
const L1_CAPACITY: usize = 10_000;
const L1_EXPIRE_SCAN_LIMIT: usize = 256;
const L2_TTL_SECS: i64 = 21_600;

pub(crate) struct ScopeCache {
    l1: DashMap<u64, CacheEntry>,
    valkey: Pool,
    http: Arc<Http>,
    l1_ttl: Duration,
    l1_capacity: usize,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    guild_id: GuildId,
    expires_at: Instant,
}

impl ScopeCache {
    pub(crate) fn new(http: Arc<Http>, valkey: Pool) -> Self {
        Self {
            l1: DashMap::new(),
            valkey,
            http,
            l1_ttl: L1_TTL,
            l1_capacity: L1_CAPACITY,
        }
    }

    pub(crate) async fn resolve_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<GuildId>, FloraError> {
        if let Some(guild_id) = self.l1_lookup(channel_id) {
            return Ok(Some(guild_id));
        }

        match self.l2_lookup_channel(channel_id).await {
            Ok(Some(guild_id)) => {
                self.insert_l1(channel_id, guild_id);
                return Ok(Some(guild_id));
            }
            Ok(None) => {}
            Err(err) => {
                warn!(target: "flora:scope_cache", channel_id = channel_id.get(), ?err, "failed to read channel scope from valkey");
            }
        }

        let channel = self
            .http
            .get_channel(channel_id.widen())
            .await
            .map_err(|err| FloraError::from_serenity_error(err, None))?;
        let guild_id = channel_guild_id(channel);
        let Some(guild_id) = guild_id else {
            return Ok(None);
        };
        self.warm_channel(channel_id, guild_id).await;
        Ok(Some(guild_id))
    }

    pub(crate) async fn resolve_webhook(
        &self,
        webhook_id: WebhookId,
    ) -> Result<Option<GuildId>, FloraError> {
        match self.l2_lookup_webhook(webhook_id).await {
            Ok(Some(guild_id)) => return Ok(Some(guild_id)),
            Ok(None) => {}
            Err(err) => {
                warn!(target: "flora:scope_cache", webhook_id = webhook_id.get(), ?err, "failed to read webhook scope from valkey");
            }
        }

        let webhook = self
            .http
            .get_webhook(webhook_id)
            .await
            .map_err(|err| FloraError::from_serenity_error(err, None))?;
        let guild_id = webhook.guild_id;
        let Some(guild_id) = guild_id else {
            return Ok(None);
        };
        self.store_webhook(webhook_id, guild_id).await;
        Ok(Some(guild_id))
    }

    pub(crate) async fn warm_channel(&self, channel_id: ChannelId, guild_id: GuildId) {
        self.insert_l1(channel_id, guild_id);
        self.store_channel(channel_id, guild_id).await;
    }

    pub(crate) async fn invalidate_channel(&self, channel_id: ChannelId) {
        self.l1.remove(&channel_id.get());
        let key = channel_key(channel_id);
        let result: Result<i64, _> = self.valkey.del(key).await;
        if let Err(err) = result {
            warn!(target: "flora:scope_cache", channel_id = channel_id.get(), ?err, "failed to invalidate channel scope in valkey");
        }
    }

    pub(crate) async fn warm_thread(&self, thread_id: ThreadId, guild_id: GuildId) {
        self.warm_channel(ChannelId::new(thread_id.get()), guild_id)
            .await;
    }

    pub(crate) async fn invalidate_thread(&self, thread_id: ThreadId) {
        self.invalidate_channel(ChannelId::new(thread_id.get()))
            .await;
    }

    fn l1_lookup(&self, channel_id: ChannelId) -> Option<GuildId> {
        let entry = self.l1.get(&channel_id.get());
        let Some(entry) = entry else {
            info!(target: "flora:scope_cache", layer = "l1", result = "miss", channel_id = channel_id.get());
            return None;
        };

        if entry.expires_at <= Instant::now() {
            drop(entry);
            self.l1.remove(&channel_id.get());
            info!(target: "flora:scope_cache", layer = "l1", result = "miss", channel_id = channel_id.get());
            return None;
        }

        let guild_id = entry.guild_id;
        info!(target: "flora:scope_cache", layer = "l1", result = "hit", channel_id = channel_id.get(), guild_id = guild_id.get());
        Some(guild_id)
    }

    async fn l2_lookup_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<GuildId>, fred::error::Error> {
        let key = channel_key(channel_id);
        let value: Option<String> = self.valkey.get(key).await?;
        let Some(value) = value else {
            info!(target: "flora:scope_cache", layer = "l2", result = "miss", channel_id = channel_id.get());
            return Ok(None);
        };
        let Ok(raw_id) = value.parse::<u64>() else {
            warn!(target: "flora:scope_cache", channel_id = channel_id.get(), value, "invalid guild id in valkey scope cache");
            return Ok(None);
        };
        let guild_id = GuildId::new(raw_id);
        info!(target: "flora:scope_cache", layer = "l2", result = "hit", channel_id = channel_id.get(), guild_id = guild_id.get());
        Ok(Some(guild_id))
    }

    async fn l2_lookup_webhook(
        &self,
        webhook_id: WebhookId,
    ) -> Result<Option<GuildId>, fred::error::Error> {
        let key = webhook_key(webhook_id);
        let value: Option<String> = self.valkey.get(key).await?;
        let Some(value) = value else {
            info!(target: "flora:scope_cache", layer = "l2", result = "miss", webhook_id = webhook_id.get());
            return Ok(None);
        };
        let Ok(raw_id) = value.parse::<u64>() else {
            warn!(target: "flora:scope_cache", webhook_id = webhook_id.get(), value, "invalid guild id in valkey scope cache");
            return Ok(None);
        };
        let guild_id = GuildId::new(raw_id);
        info!(target: "flora:scope_cache", layer = "l2", result = "hit", webhook_id = webhook_id.get(), guild_id = guild_id.get());
        Ok(Some(guild_id))
    }

    fn insert_l1(&self, channel_id: ChannelId, guild_id: GuildId) {
        let entry = CacheEntry {
            guild_id,
            expires_at: Instant::now() + self.l1_ttl,
        };
        self.l1.insert(channel_id.get(), entry);
        self.enforce_l1_cap();
    }

    fn enforce_l1_cap(&self) {
        let mut len = self.l1.len();
        if len <= self.l1_capacity {
            return;
        }

        let now = Instant::now();
        let mut expired_keys = Vec::new();
        let mut scanned = 0usize;
        for entry in self.l1.iter() {
            if scanned >= L1_EXPIRE_SCAN_LIMIT {
                break;
            }
            scanned += 1;
            if entry.value().expires_at <= now {
                expired_keys.push(*entry.key());
            }
        }

        for key in expired_keys {
            self.l1.remove(&key);
        }

        len = self.l1.len();
        if len <= self.l1_capacity {
            return;
        }

        let mut to_remove = Vec::new();
        let mut excess = len - self.l1_capacity;
        for entry in self.l1.iter() {
            to_remove.push(*entry.key());
            excess -= 1;
            if excess == 0 {
                break;
            }
        }

        for key in to_remove {
            self.l1.remove(&key);
        }
    }

    async fn store_channel(&self, channel_id: ChannelId, guild_id: GuildId) {
        let key = channel_key(channel_id);
        let value = guild_id.get().to_string();
        let result = self
            .valkey
            .set::<(), _, _>(key, value, Some(Expiration::EX(L2_TTL_SECS)), None, false)
            .await;
        if let Err(err) = result {
            warn!(target: "flora:scope_cache", channel_id = channel_id.get(), guild_id = guild_id.get(), ?err, "failed to write channel scope to valkey");
        }
    }

    async fn store_webhook(&self, webhook_id: WebhookId, guild_id: GuildId) {
        let key = webhook_key(webhook_id);
        let value = guild_id.get().to_string();
        let result = self
            .valkey
            .set::<(), _, _>(key, value, Some(Expiration::EX(L2_TTL_SECS)), None, false)
            .await;
        if let Err(err) = result {
            warn!(target: "flora:scope_cache", webhook_id = webhook_id.get(), guild_id = guild_id.get(), ?err, "failed to write webhook scope to valkey");
        }
    }
}

fn channel_key(channel_id: ChannelId) -> String {
    format!("flora:scope:ch:{}", channel_id.get())
}

fn webhook_key(webhook_id: WebhookId) -> String {
    format!("flora:scope:wh:{}", webhook_id.get())
}

fn channel_guild_id(channel: Channel) -> Option<GuildId> {
    channel.guild_id()
}
