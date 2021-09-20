

use std::collections::HashMap;

/// the `KvStore` stores string key/value pairs
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk
///
/// Example:
/// ```rust
///  # use kvs::KvStore;
///  let mut store = KvStore::new();
///  store.set("key".to_owned(), "value".to_owned());
///  let val = store.get("key".to_owned());
///  assert_eq!(val, Some("value".to_owned()));
/// ```
pub struct KvStore {
    map: HashMap<String, String>,
}

impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KvStore {
    /// Creates a `KvStore`
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// Sets the value of a string key to a string
    ///
    /// If the key already exisits, will replace the old key
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Gets the key value
    ///
    /// Return `None` if the key not exist.
    pub fn get(&self, key: String) -> Option<String> {
        return self.map.get(&key).cloned();
    }

    /// remove the key
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}