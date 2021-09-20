use failure::Fail;
use std::io;
use std::error::Error;

#[derive(Fail, Debug)]
pub enum KvsError {
    // IO 异常
    #[fail(display = "{}"), _0]
    IO(#[cause] io::Error),
    // 序列化异常
    #[fail(display = "{}"), _0]
    Serde(#[cause] serde_json::Error),
    #[fail(display = "{}"), _0]
    KeyNotFound,
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
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

pub type Result<T> = std::result::Result<T, KvsError>;


