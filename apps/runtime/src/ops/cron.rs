use chrono::{DateTime, Utc};
use croner::Cron;
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use parking_lot::Mutex;
use serde::Deserialize;
use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr, sync::Arc};

pub const CRON_EVENT_PREFIX: &str = "__cron:";

pub struct CronJob {
    pub guild_id: Option<String>,
    pub name: String,
    pub event_name: String,
    pub schedule: Cron,
    pub next_run: DateTime<Utc>,
    pub skip_if_running: bool,
    pub is_running: bool,
}

impl Clone for CronJob {
    fn clone(&self) -> Self {
        Self {
            guild_id: self.guild_id.clone(),
            name: self.name.clone(),
            event_name: self.event_name.clone(),
            schedule: Cron::from_str(&self.schedule.pattern.to_string()).unwrap(),
            next_run: self.next_run,
            skip_if_running: self.skip_if_running,
            is_running: self.is_running,
        }
    }
}

pub struct CronRegistry {
    pub jobs: HashMap<Option<String>, Vec<CronJob>>,
    pub max_per_guild: usize,
}

impl CronRegistry {
    pub fn new(max_per_guild: usize) -> Self {
        Self {
            jobs: HashMap::new(),
            max_per_guild,
        }
    }

    pub fn clear_guild(&mut self, guild_id: &str) {
        self.jobs.remove(&Some(guild_id.to_owned()));
    }
}

pub type SharedCronRegistry = Arc<Mutex<CronRegistry>>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterCronArgs {
    name: String,
    expr: String,
    #[serde(default)]
    skip_if_running: bool,
}

#[op2]
pub fn op_register_cron(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RegisterCronArgs,
) -> Result<(), JsErrorBox> {
    let state = state.borrow();
    let guild_id: Option<String> = state.try_borrow::<String>().cloned();
    let registry = state.borrow::<SharedCronRegistry>().clone();

    let schedule = Cron::from_str(&args.expr).map_err(|e| {
        JsErrorBox::generic(format!("invalid cron expression '{}': {e}", args.expr))
    })?;

    let mut reg = registry.lock();
    let max_per_guild = reg.max_per_guild;
    let jobs = reg.jobs.entry(guild_id.clone()).or_default();

    if jobs.len() >= max_per_guild {
        return Err(JsErrorBox::generic(
            "max cron jobs exceeded for this guild/runtime",
        ));
    }

    for job in jobs.iter() {
        if job.name == args.name {
            return Err(JsErrorBox::generic(format!(
                "cron job with name '{}' already exists",
                args.name
            )));
        }
    }

    let next_run = schedule
        .find_next_occurrence(&Utc::now(), false)
        .map_err(|e| {
            JsErrorBox::generic(format!("cron schedule has no future occurrences: {e}"))
        })?;

    jobs.push(CronJob {
        guild_id,
        name: args.name.clone(),
        event_name: format!("{}{}", CRON_EVENT_PREFIX, args.name),
        schedule,
        next_run,
        skip_if_running: args.skip_if_running,
        is_running: false,
    });

    Ok(())
}
