use rocksdb::DB;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

pub(crate) const MAX_DB_CACHE_SIZE: usize = 64;

pub(crate) struct BoundedCache {
    map: HashMap<String, Arc<DB>>,
    order: VecDeque<String>,
    capacity: usize,
}

impl BoundedCache {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<&Arc<DB>> {
        self.map.get(key)
    }

    pub(crate) fn insert(&mut self, key: String, db: Arc<DB>) {
        if self.map.len() >= self.capacity
            && let Some(oldest) = self.order.pop_back()
        {
            self.map.remove(&oldest);
        }
        self.order.push_front(key.clone());
        self.map.insert(key, db);
    }

    pub(crate) fn remove(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.map.remove(key);
    }
}
