

use std::collections::{HashMap, BTreeMap};
use std::path::{PathBuf, Path};
use std::io::{BufReader, Read, Seek, SeekFrom, IoSliceMut, Bytes, Chain, Take, Write, BufWriter, IoSlice};
use crate::{KvsError, Result};
use std::fmt::Arguments;
use std::fs::{File, OpenOptions, read};
use std::ffi::OsStr;
use serde_json::Deserializer;
use serde::{Deserialize, Serialize};
use std::ops::Range;


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
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) +1;

        let writer = new_log_file(&path, current_gen, &mut readers)?;

        Ok(KvStore{
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
        if let Command::Set { Key, ..} = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(Key, (self.current_gen, pos..self.writer.pos).into()) {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    /// Gets the key value
    ///
    /// Return `None` if the key not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self.readers.get_mut(&cmd_pos.gen).expect("Cannot find log reader");
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);
            if let Command::Set { Value, ..} = serde_json::from_reader(cmd_reader)? {
                Ok(Some(Value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// remove the key
    pub fn remove(&mut self, key: String) -> Result<()>{
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { Key} = cmd {
                let old_cmd = self.index.remove(&Key).expect("key not found");
                self.uncompacted += old_cmd.len;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }


    // Clears stale entries in  the log
    pub fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compact file
        let compaction_gen = self.current_gen + 1;
        self.current_gen +=2;
        // 保证写入的正常
        self.writer = self.new_log_file(self.current_gen)?;
        // 后期其实是可以异步操作的
        let mut compaction_writer = self.new_log_file(compaction_gen)?;

        let mut new_pos = 0; // pos in the new log file
        for cmd_pos in &mut self.index.values_mut() {
            // 根据gen找到对应的reader
            let reader = self.readers
                .get_mut(&cmd_pos.gen)
                .expect("Cannot find log reader");
            // 寻址
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }
            // 读出一行
            let mut entry_reader = reader.take(cmd_pos.len);
            // 写入到compaction 中
            let len = std::io::copy(&mut entry_reader, &mut compaction_writer)?;
            // 修改值
            *cmd_pos = (compaction_gen, new_pos..new_pos + len).into();
            // 写入位置累加
            new_pos += len;
        }

        compaction_writer.flush();
        // 删除无用的文件
        let stable_gens: Vec<_> = self.readers.keys().filter(|&&gen|gen < compaction_gen).cloned().collect();

        for stable_gen in stable_gens {
            self.readers.remove(&stable_gen);
            std::fs::remove_file(log_path(&self.path, stable_gen))?;
        }
        self.uncompacted = 0;

        Ok(())
    }

    fn new_log_file(&mut self, gen: u64) -> Result<BufWriterWithPos<File>> {
        new_log_file(&self.path, gen, &mut self.readers)
    }

}

fn load(
    gen: u64,
    reader: &mut BufReaderWithPo<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set {Key, ..} => {
                if let Some(old_cmd) = index.insert(Key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len;
                }
            }
            Command:: Remove { Key} => {
                if let Some(old_cmd) = index.remove(&Key) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
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

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
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