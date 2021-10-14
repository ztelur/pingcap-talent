use failure::Fail;
use std::io;
use std::error::Error;
use std::string::FromUtf8Error;

#[derive(Fail, Debug)]
pub enum KvsError {
    // IO 异常
    #[fail(display = "{}", _0)]
    IO(#[cause] io::Error),
    // 序列化异常
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
    #[fail(display = "UTF-8 error : {}", _0)]
    Utf8(#[cause] FromUtf8Error),
    #[fail(display = "sled error: {}", _0)]
    Sled(#[cause] sled::Error),
    #[fail(display = "{}"), _0]
    StringError(String),
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::IO(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(err: FromUtf8Error) -> Self {
        KvsError::Utf8(err)
    }
}

impl From<sled::Error> for KvsError {
    fn from(err: sled::Error) -> Self {
        KvsError::Sled(err)
    }
}


pub type Result<T> = std::result::Result<T, KvsError>;


