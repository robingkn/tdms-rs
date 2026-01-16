pub mod api;
pub mod error;
pub mod format;
pub mod io;
pub mod model;

pub use api::reader::{TdmsChannel, TdmsFile, TdmsGroup, TdmsSlice};
pub use api::writer::{TdmsWriter, WriterChannel, WriterGroup};
pub use error::{Result, TdmsError};
pub use model::datatypes::{DataType, PropertyValue};

// Backward compatibility aliases
pub type TdmsDType = DataType;
