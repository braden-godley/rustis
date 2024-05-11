use std::collections::HashMap;
use std::time::{Instant, Duration};

struct KvStore {
    map: HashMap<String, Val>,
    last_check_time: Instant,
}

struct Val {
    val: String,
    ttl: Option<Duration>,
    created_at: Instant,
}

impl Val {
    pub fn new(val: &str, ttl: Option<u64>) -> Self {
        Val {
            val: val.to_string(),
            ttl: ttl.map_or(None, |ttl| Some(Duration::from_secs(ttl))),
            created_at: Instant::now(),
        }
    }
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            map: HashMap::new(),
            last_check_time: Instant::now(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let val = Val::new(value, None);
        self.map.insert(key.to_string(), val);
    }

    pub fn setex(&mut self, key: &str, value: &str, ttl: u64) {
        let val = Val::new(value, Some(ttl));
        self.map.insert(key.to_string(), val);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let val = self.map.get(&key.to_string());

        if let Some(val) = val {
            if let Some(ttl) = val.ttl {
                let expired = val.created_at.elapsed() < ttl;
                if expired {
                    None
                } else {
                    Some(val.val.clone())
                }
            } else {
                None
            }
        } else {
            None
        }

    }

    pub fn ttl(&self, key: &str) -> Option<u64> {
        let val = self.map.get(&key.to_string());

        if let Some(val) = val {
            if let Some(ttl) = val.ttl {
                let expired = val.created_at.elapsed() < ttl;
                if expired {
                    None
                } else {
                    Some(ttl.as_secs())
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn collect_garbage(&mut self) {
        let mut old_keys = Vec::new();
        for (key, val) in &self.map {
            if let Some(ttl) = val.ttl {
                if val.created_at.elapsed() >= ttl {
                    old_keys.push(key.to_string());
                }
            }
        }

        for old_key in old_keys {
            self.map.remove(&old_key);
        }
        self.last_check_time = Instant::now();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set() {
        let mut kv = KvStore::new();

        kv.set("test", "value");

        let result = kv.get("test");

        if let Some(val) = result {
            assert_eq!(&val[..], "value");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_setex() {
        let mut kv = KvStore::new();

        kv.setex("test", "value", 10);

        let result = kv.get("test");

        if let Some(val) = result {
            assert_eq!(&val[..], "value");
        } else {
            assert!(false);
        }

        let ttl = kv.ttl("test");

        assert_eq!(ttl.is_some(), true);
        assert_eq!(ttl.unwrap() >= 9, true);
    }
}
