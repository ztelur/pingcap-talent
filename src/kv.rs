

use std::collections::{HashMap, BTreeMap};
use std::path::{PathBuf, Path};
use std::io::{BufReader, Read, Seek, SeekFrom, IoSliceMut, Bytes, Chain, Take, Write, BufWriter, IoSlice};
use crate::{KvsError, Result};
use std::fmt::Arguments;
use std::fs::{File, OpenOptions};
use std::ffi::OsStr;


const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// the `KvStore` stores string key/value pairs
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk
///
/// Example:
/// ```rust
/// ```
pub struct KvStore {
    // log and other data directory
    path: PathBuf,
    readers: HashMap<u64, BufReaderWithPo<File>>,
    writer: BufWriterWithPos<File>,
    current_gen: u64,
    index: BTreeMap<String, CommandPos>,
    uncompacted: u64,
}

impl KvStore {
    /// Creates a `KvStore`
    ///
    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();

        std::fs::create_dir_all(&path);

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        let gen_list = sort_gen_list(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPo::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &mut index);
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) +1;

        let writer = new_log_file(&path, current_gen, &mut readers)?;

        OK(KvStore{
            path,
            readers,
            writer,
            current_gen,
            index,
            uncompacted,
        })
    }

    /// Sets the value of a string key to a string
    ///
    /// If the key already exisits, will replace the old key
    pub fn set(&mut self, key: String, value: String) -> Result<()>{
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd);
        self.writer.flush()?;
        if let Command::Set { key, ..} = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(key, (self.current_gen, pos..self.writer.pos).into()) {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.com
        }
        Ok(())
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


#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { Key:String, Value:String},
    Remove { Key:String},
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { Key: key, Value: value}
    }
    fn remove(key: String) -> Command {
        Command::Remove { Key : key}
    }
}




// show the position and length of a json-serialized command in the log
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
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
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPo<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos).unwrap();
        Ok(self.pos)
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
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}



fn sort_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = std::fs::read_dir(&path)?
        .flat_map(|res| -> Result<_>{ Ok(res?.path())})
        .filter(|path1| path1.is_file() && path1.extension() == Some("log".as_ref()))
        .flat_map(|path|{
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();

    gen_list.sort_unstable();
    Ok(gen_list)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log",gen))
}

fn new_log_file(dir: &Path, gen: u64, readers: &mut HashMap<u64, BufReaderWithPo<File>>,) -> Result<BufWriterWithPos<File>> {
    let path = log_path(dir, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    readers.insert(gen, BufReaderWithPo::new(File::open(&path)?)?);
    Ok(writer)
}