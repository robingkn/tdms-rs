use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TdmsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid TDMS format: {0}")]
    InvalidFormat(String),

    #[error("Invalid TDMS signature")]
    InvalidSignature,

    #[error("Unknown version: {0}")]
    UnknownVersion(u32),

    #[error("Group '{0}' not found")]
    GroupNotFound(String),

    #[error("Channel '{0}' not found in group '{1}'")]
    ChannelNotFound(String, String),

    #[error("Unsupported data type: {0}")]
    UnsupportedDataType(u32),

    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Duplicate name: {0}")]
    DuplicateName(String),

    #[error("String encoding error")]
    StringEncoding,

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("String formatting error: {0}")]
    StringFormatting(String),
}

pub type Result<T> = std::result::Result<T, TdmsError>;
