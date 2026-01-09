
use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum TdmsError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid TDMS Signature")]
    InvalidSignature,
    #[error("Unknown Version: {0}")]
    UnknownVersion(u32),
    #[error("String encoding error")]
    StringEncoding,
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("String formatting error: {0}")]
    StringFormatting(String),
}

pub type Result<T> = std::result::Result<T, TdmsError>;
