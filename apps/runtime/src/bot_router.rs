use parking_lot::RwLock;
use serenity::http::Http;
use std::{collections::HashMap, sync::Arc};

/// Routes guild-scoped runtime actions/events to the correct bot client.
#[derive(Clone)]
pub struct GuildBotRouter {
    fallback_bot_user_id: String,
    fallback_http: Arc<Http>,
    fallback_enabled: bool,
    guild_to_bot: Arc<RwLock<HashMap<String, String>>>,
    bot_http: Arc<RwLock<HashMap<String, Arc<Http>>>>,
}

impl GuildBotRouter {
    pub fn new(
        fallback_bot_user_id: String,
        fallback_http: Arc<Http>,
        fallback_enabled: bool,
    ) -> Self {
        Self {
            fallback_bot_user_id,
            fallback_http,
            fallback_enabled,
            guild_to_bot: Arc::new(RwLock::new(HashMap::new())),
            bot_http: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn fallback_bot_user_id(&self) -> &str {
        &self.fallback_bot_user_id
    }

    pub fn register_bot_http(&self, bot_user_id: String, http: Arc<Http>) {
        self.bot_http.write().insert(bot_user_id, http);
    }

    pub fn set_guild_binding(&self, guild_id: String, bot_user_id: String) {
        self.guild_to_bot.write().insert(guild_id, bot_user_id);
    }

    pub fn clear_guild_binding(&self, guild_id: &str) {
        self.guild_to_bot.write().remove(guild_id);
    }

    pub fn clear_all_guild_bindings(&self) {
        self.guild_to_bot.write().clear();
    }

    pub fn target_bot_for_guild(&self, guild_id: &str) -> Option<String> {
        if let Some(bound_bot) = self.guild_to_bot.read().get(guild_id) {
            return Some(bound_bot.clone());
        }

        if self.fallback_enabled {
            Some(self.fallback_bot_user_id.clone())
        } else {
            None
        }
    }

    pub fn http_for_guild(&self, guild_id: Option<&str>) -> Option<Arc<Http>> {
        let Some(guild_id) = guild_id else {
            return Some(self.fallback_http.clone());
        };

        let target_bot = self.target_bot_for_guild(guild_id)?;
        if target_bot == self.fallback_bot_user_id {
            return Some(self.fallback_http.clone());
        }

        self.bot_http.read().get(&target_bot).cloned()
    }

    pub fn should_handle_event(&self, source_bot_user_id: &str, guild_id: &str) -> bool {
        self.target_bot_for_guild(guild_id)
            .as_deref()
            .is_some_and(|target| target == source_bot_user_id)
    }
}
