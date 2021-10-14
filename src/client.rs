use crate::{KvsError, Result};


use serde_json::de::IoRead;
use std::io::{BufRead, BufWriter, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use serde_json::Deserializer;
use crate::common::{Request, GetResponse, RemoveResponse};

pub struct KvsClient {
    reader: Deserializer<IoRead<BufRead<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}


impl KvsClient {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;

        Ok(KvsClient {
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)),
            writer: BufWriter::new(tcp_writer),
        })
    }


    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get {key})?;
        self.writer.flush()?;
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Set { key, value})?;
        self.writer.flush()?;
        let resp = GetResponse::deserialize(&mut self.reader)?;

        match resp {
            GetResponse::Ok(val) => Ok(value),
            GetResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }

    }



    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self, &Request::Remove {key})?;
        self.writer.flush()?;

        let resp = RemoveResponse::deserialize(&mut self.reader)?;
        match resp {
            RemoveResponse::Ok(_) => Ok(()),
            RemoveResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }





}