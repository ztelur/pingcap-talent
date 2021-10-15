mod kvs;
mod sled;

use crate::Result;

pub trait KvsEngine {
    fn set(&self, key: String, value: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&self, key: String) -> Result<()>;
}

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
