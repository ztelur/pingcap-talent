

use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{BufReader, Read, Seek, SeekFrom, IoSliceMut, Initializer, Bytes, Chain, Take, Write, BufWriter, IoSlice};
use crate::{KvsError, Result};
use std::fmt::Arguments;

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
    // log and other data directory
    path: PathBuf,
    readers: HashMap<u64, BufReader>
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

struct BufReaderWithPo <R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPo<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(
            BufReaderWithPo {
                reader: BufReader::new(inner),
                pos,
            }
        )
    }
}

impl<R: Read + Seek> Read for BufReaderWithPo<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf);
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPo<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos).unwrap();
        OK(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }

}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        OK(self.pos);
    }
}
