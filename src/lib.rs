//! # tdms-rs
//!
//! A pure Rust library for reading and writing National Instruments TDMS (Technical Data Management Streaming) files.
//!
//! Version: 2.0.0
pub mod api;
pub mod error;
mod format;
mod io;
mod model;

pub use api::reader::{TdmsChannel, TdmsFile, TdmsGroup};
pub use api::writer::{TdmsWriter, WriterChannel, WriterGroup};
pub use error::{Result, TdmsError};
pub use model::datatypes::{DataType, PropertyValue};
