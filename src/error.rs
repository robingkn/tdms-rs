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

    #[error("Type mismatch")]
    TypeMismatch,

    #[error("Alignment error")]
    AlignmentError,

    #[error("Invalid range: {0}..{1} (len={2})")]
    InvalidRange(usize, usize, usize),

    #[error("Writer already closed")]
    WriterClosed,

    #[error("Compression not supported: {0}")]
    CompressionNotSupported(String),

    #[error("Property too large: {0} bytes")]
    PropertyTooLarge(usize),

    #[error("File closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, TdmsError>;
