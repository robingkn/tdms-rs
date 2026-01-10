//! # tdms-rs
//!
//! A pure Rust library for reading and writing National Instruments TDMS
//! (Technical Data Management Streaming) files with full format support and excellent performance.
//!
//! ## Features
//!
//! - **Complete TDMS Support**: Read and write all TDMS data types and structures
//! - **High Performance**: Zero-copy parsing and efficient memory usage  
//! - **Type Safety**: Rust's type system prevents common data handling errors
//! - **Binary Compatibility**: Output files work with National Instruments software
//! - **Production Ready**: Comprehensive test coverage with 24+ test scenarios
//! - **Pure Rust**: Minimal external dependencies
//!
//! ## Quick Start
//!
//! ### Reading TDMS Files
//!
//! ```no_run
//! use tdms_rs::TdmsFile;
//! use std::path::Path;
//!
//! // Load a TDMS file
//! let file = TdmsFile::load(Path::new("data.tdms"))?;
//!
//! // Iterate through groups and channels
//! for (group_name, group) in &file.groups {
//!     println!("Group: {}", group_name);
//!     for (channel_name, channel) in &group.channels {
//!         if let Some(data) = &channel.data {
//!             println!("  Channel {}: {} samples", channel_name, data.len());
//!             
//!             // Access typed data
//!             match data {
//!                 tdms_rs::TdmsData::Double(values) => {
//!                     let avg = values.iter().sum::<f64>() / values.len() as f64;
//!                     println!("    Average: {:.2}", avg);
//!                 },
//!                 tdms_rs::TdmsData::I32(values) => {
//!                     println!("    Range: {} to {}",
//!                         values.iter().min().unwrap(),
//!                         values.iter().max().unwrap());
//!                 },
//!                 _ => println!("    Other data type"),
//!             }
//!         }
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Writing TDMS Files
//!
//! ```no_run
//! use tdms_rs::{TdmsFileWriter, TdmsData, PropertyValue};
//!
//! // Create a new TDMS file
//! let mut writer = TdmsFileWriter::new("output.tdms");
//!
//! // Add file-level properties
//! writer.add_property("Author", PropertyValue::String("Rust App".into()))?;
//!
//! // Create groups and channels
//! let group = writer.add_group("Sensors")?;
//! group.add_channel("Temperature", TdmsData::Double(vec![20.1, 21.5, 22.3]))?;
//! group.add_channel("Pressure", TdmsData::I32(vec![1013, 1015, 1012]))?;
//!
//! // Add channel properties
//! let voltage_channel = group.add_channel("Voltage", TdmsData::Double(vec![1.1, 2.2, 3.3]))?;
//! voltage_channel.add_property("wf_unit_string", PropertyValue::String("V".into()))?;
//!
//! // Write the file
//! writer.write()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Error Handling
//!
//! ```no_run
//! use tdms_rs::TdmsFile;
//! use std::path::Path;
//!
//! match TdmsFile::load(Path::new("data.tdms")) {
//!     Ok(file) => println!("Loaded {} groups", file.groups.len()),
//!     Err(e) => eprintln!("Failed to load TDMS file: {}", e),
//! }
//! ```
//!
//! ## Supported Data Types
//!
//! All TDMS data types are fully supported with Rust type safety:
//!
//! | TDMS Type | Rust Type | Description |
//! |-----------|-----------|-------------|
//! | I8-I64    | `i8`-`i64` | Signed integers |
//! | U8-U64    | `u8`-`u64` | Unsigned integers |
//! | Float     | `f32`     | 32-bit floating point |
//! | Double    | `f64`     | 64-bit floating point |
//! | String    | `String`  | UTF-8 encoded text |
//! | Boolean   | `bool`    | True/false values |
//! | TimeStamp | `(i64, u64)` | TDMS timestamp format |
//!
//! ## Performance Guarantees
//!
//! - **Zero-copy parsing** where possible for efficient memory usage
//! - **Streaming reads** handle large files without loading everything into memory
//! - **Binary compatibility** with National Instruments TDMS readers
//! - **Deterministic output** ensures consistent file generation

pub mod channel;
pub mod datatypes;
pub mod error;
pub mod metadata;
pub mod reader;
pub mod segment;
pub mod utils;
pub mod writer;

use crate::error::Result;
use crate::reader::TdmsReader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use indexmap::IndexMap;

/// Common TDMS property names as constants to avoid string duplication.
///
/// These constants represent well-known property names used in TDMS files,
/// particularly for waveform data and National Instruments-specific metadata.
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::{TdmsFile, properties};
/// use std::path::Path;
///
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
/// if let Some(channel) = file.get_channel("Group", "Channel") {
///     // Use constants instead of magic strings
///     if let Some(unit) = channel.get_string_property(properties::UNIT_STRING) {
///         println!("Unit: {}", unit);
///     }
///     if let Some(increment) = channel.get_double_property(properties::INCREMENT) {
///         println!("Sample interval: {} seconds", increment);
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub mod properties {
    /// Waveform unit string property ("wf_unit_string")
    pub const UNIT_STRING: &str = "wf_unit_string";

    /// Waveform time increment property ("wf_increment")
    pub const INCREMENT: &str = "wf_increment";

    /// Waveform start time property ("wf_start_time")
    pub const START_TIME: &str = "wf_start_time";

    /// Waveform start offset property ("wf_start_offset")
    pub const START_OFFSET: &str = "wf_start_offset";

    /// Waveform sample count property ("wf_samples")
    pub const SAMPLES: &str = "wf_samples";

    /// National Instruments array column property ("NI_ArrayColumn")
    pub const NI_ARRAY_COLUMN: &str = "NI_ArrayColumn";

    /// National Instruments channel length property ("NI_ChannelLength")
    pub const NI_CHANNEL_LENGTH: &str = "NI_ChannelLength";

    /// Description property ("Description")
    pub const DESCRIPTION: &str = "Description";

    /// Sensor type property ("Sensor_Type")
    pub const SENSOR_TYPE: &str = "Sensor_Type";

    /// Calibration date property ("Calibration_Date")
    pub const CALIBRATION_DATE: &str = "Calibration_Date";
}

/// A TDMS file containing groups and channels with their associated data and properties.
///
/// TDMS (Technical Data Management Streaming) is a file format developed by National Instruments
/// for storing measurement data. A TDMS file has a hierarchical structure:
/// - File (root level)
/// - Groups (containers for related channels)  
/// - Channels (individual data streams with properties and data)
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::TdmsFile;
/// use std::path::Path;
///
/// // Load a TDMS file
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
///
/// // Access file-level properties
/// for (prop_name, prop_value) in &file.properties {
///     println!("File property {}: {:?}", prop_name, prop_value);
/// }
///
/// // Iterate through groups and channels
/// for (group_name, group) in &file.groups {
///     println!("Group: {}", group_name);
///     for (channel_name, channel) in &group.channels {
///         if let Some(data) = &channel.data {
///             let sample_count = match data {
///                 tdms_rs::TdmsData::Double(v) => v.len(),
///                 tdms_rs::TdmsData::I32(v) => v.len(),
///                 _ => 0, // Handle other types as needed
///             };
///             println!("  Channel {}: {} samples", channel_name, sample_count);
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsFile {
    /// Properties (metadata) associated with the file itself.
    ///
    /// File-level properties contain metadata about the entire TDMS file, such as:
    /// - Creation time and author information
    /// - File format version
    /// - Custom application-specific metadata
    pub properties: IndexMap<String, PropertyValue>,

    /// Groups contained in this TDMS file, indexed by group name.
    ///
    /// Group names are the path components from the TDMS file structure.
    /// For example, a path like `/'Sensors'/'Temperature'` creates a group named "Sensors"
    /// containing a channel named "Temperature".
    pub groups: IndexMap<String, TdmsGroup>,
}

pub use crate::datatypes::{PropertyValue, TdmsData};
pub use crate::writer::{TdmsChannelWriter, TdmsFileWriter, TdmsGroupWriter};

/// A group within a TDMS file containing related channels and group-level properties.
///
/// Groups serve as containers for organizing related measurement channels.
/// They can have their own properties (metadata) and contain multiple channels.
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::TdmsFile;
/// use std::path::Path;
///
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
///
/// if let Some(group) = file.groups.get("Sensors") {
///     println!("Group has {} properties", group.properties.len());
///     println!("Group has {} channels", group.channels.len());
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsGroup {
    /// Properties (metadata) associated with this group.
    ///
    /// Properties are key-value pairs that provide metadata about the group.
    /// Common properties might include creation time, description, or custom metadata.
    pub properties: IndexMap<String, PropertyValue>,

    /// Channels contained in this group, indexed by channel name.
    ///
    /// Each channel represents a single data stream (e.g., a sensor reading over time).
    pub channels: IndexMap<String, TdmsChannel>,
}

/// A channel within a TDMS group containing data and channel-level properties.
///
/// Channels represent individual data streams, such as sensor readings, timestamps,
/// or calculated values. Each channel has associated metadata (properties) and
/// may contain actual measurement data.
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::{TdmsFile, TdmsData};
/// use std::path::Path;
///
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
///
/// if let Some(group) = file.groups.get("Sensors") {
///     if let Some(channel) = group.channels.get("Temperature") {
///         // Access channel data
///         match &channel.data {
///             Some(TdmsData::Double(values)) => {
///                 println!("Temperature readings: {} samples", values.len());
///                 if let Some(first) = values.first() {
///                     println!("First reading: {:.2}°C", first);
///                 }
///             },
///             Some(other) => println!("Unexpected data type: {:?}", other),
///             None => println!("No data in channel"),
///         }
///         
///         // Access channel properties
///         if let Some(unit) = channel.properties.get("wf_unit_string") {
///             println!("Unit: {:?}", unit);
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsChannel {
    /// Properties (metadata) associated with this channel.
    ///
    /// Channel properties contain metadata about the data stream, such as:
    /// - `wf_unit_string`: Physical unit of measurement (e.g., "°C", "V", "Hz")
    /// - `wf_increment`: Time increment between samples for waveform data
    /// - `wf_start_time`: Start time for waveform data
    /// - `wf_samples`: Number of samples in the waveform
    /// - Custom properties defined by the application
    pub properties: IndexMap<String, PropertyValue>,

    /// The actual measurement data for this channel.
    ///
    /// Data is `None` if the channel contains only metadata (no actual measurements).
    /// When present, data is stored in a type-safe enum that preserves the original
    /// TDMS data type (integers, floats, strings, timestamps, etc.).
    pub data: Option<TdmsData>,
}

impl Default for TdmsGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl TdmsGroup {
    /// Create a new empty group.
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
            channels: IndexMap::new(),
        }
    }

    /// Get an iterator over channels in this group.
    pub fn iter_channels(&self) -> impl Iterator<Item = (&str, &TdmsChannel)> {
        self.channels.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get a channel by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(group) = file.group("Sensors") {
    ///     if let Some(channel) = group.channel("Temperature") {
    ///         println!("Found temperature channel");
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn channel(&self, name: &str) -> Option<&TdmsChannel> {
        self.channels.get(name)
    }
}

impl Default for TdmsChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl TdmsChannel {
    /// Create a new empty channel.
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
            data: None,
        }
    }
    /// Get channel data as f64 slice if the data type is Double.
    pub fn as_f64(&self) -> Option<&[f64]> {
        match &self.data {
            Some(TdmsData::Double(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as f32 slice if the data type is Float.
    pub fn as_f32(&self) -> Option<&[f32]> {
        match &self.data {
            Some(TdmsData::Float(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as i32 slice if the data type is I32.
    pub fn as_i32(&self) -> Option<&[i32]> {
        match &self.data {
            Some(TdmsData::I32(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as String slice if the data type is String.
    pub fn as_string(&self) -> Option<&[String]> {
        match &self.data {
            Some(TdmsData::String(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as i8 slice if the data type is I8.
    pub fn as_i8(&self) -> Option<&[i8]> {
        match &self.data {
            Some(TdmsData::I8(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as i16 slice if the data type is I16.
    pub fn as_i16(&self) -> Option<&[i16]> {
        match &self.data {
            Some(TdmsData::I16(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as i64 slice if the data type is I64.
    pub fn as_i64(&self) -> Option<&[i64]> {
        match &self.data {
            Some(TdmsData::I64(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as u8 slice if the data type is U8.
    pub fn as_u8(&self) -> Option<&[u8]> {
        match &self.data {
            Some(TdmsData::U8(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as u16 slice if the data type is U16.
    pub fn as_u16(&self) -> Option<&[u16]> {
        match &self.data {
            Some(TdmsData::U16(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as u32 slice if the data type is U32.
    pub fn as_u32(&self) -> Option<&[u32]> {
        match &self.data {
            Some(TdmsData::U32(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as u64 slice if the data type is U64.
    pub fn as_u64(&self) -> Option<&[u64]> {
        match &self.data {
            Some(TdmsData::U64(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as bool slice if the data type is Boolean.
    pub fn as_bool(&self) -> Option<&[bool]> {
        match &self.data {
            Some(TdmsData::Boolean(values)) => Some(values),
            _ => None,
        }
    }

    /// Get channel data as timestamp slice if the data type is TimeStamp.
    pub fn as_timestamps(&self) -> Option<&[(i64, u64)]> {
        match &self.data {
            Some(TdmsData::TimeStamp(values)) => Some(values),
            _ => None,
        }
    }

    /// Convert any numeric data to f64 vector.
    /// Returns None if the data is not numeric.
    pub fn as_numeric(&self) -> Option<Vec<f64>> {
        match &self.data {
            Some(TdmsData::Double(values)) => Some(values.clone()),
            Some(TdmsData::Float(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::I8(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::I16(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::I32(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::I64(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::U8(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::U16(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::U32(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            Some(TdmsData::U64(values)) => Some(values.iter().map(|&v| v as f64).collect()),
            _ => None,
        }
    }

    /// Get the number of data samples in this channel.
    pub fn data_len(&self) -> usize {
        match &self.data {
            Some(data) => data.len(),
            None => 0,
        }
    }

    /// Get a human-readable name for the data type.
    pub fn data_type_name(&self) -> Option<&'static str> {
        match &self.data {
            Some(data) => Some(data.type_name()),
            None => None,
        }
    }

    // Property helpers for common TDMS properties

    /// Get the unit string property (wf_unit_string).
    pub fn unit(&self) -> Option<&str> {
        self.get_string_property("wf_unit_string")
    }

    /// Get the increment property (wf_increment) as f64.
    pub fn increment(&self) -> Option<f64> {
        self.get_double_property("wf_increment")
    }

    /// Get the start time property (wf_start_time) as f64.
    pub fn start_time(&self) -> Option<f64> {
        self.get_double_property("wf_start_time")
    }

    /// Get a string property value by name.
    pub fn get_string_property(&self, name: &str) -> Option<&str> {
        match self.properties.get(name) {
            Some(PropertyValue::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Get a double property value by name.
    pub fn get_double_property(&self, name: &str) -> Option<f64> {
        match self.properties.get(name) {
            Some(PropertyValue::Double(d)) => Some(*d),
            Some(PropertyValue::Float(f)) => Some(*f as f64),
            _ => None,
        }
    }

    /// Get an i32 property value by name.
    pub fn get_i32_property(&self, name: &str) -> Option<i32> {
        match self.properties.get(name) {
            Some(PropertyValue::I32(i)) => Some(*i),
            Some(PropertyValue::I16(i)) => Some(*i as i32),
            Some(PropertyValue::I8(i)) => Some(*i as i32),
            _ => None,
        }
    }

    /// Get the sample count from wf_samples property.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(channel) = file.get_channel("Group", "Channel") {
    ///     if let Some(count) = channel.sample_count() {
    ///         println!("Expected {} samples", count);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sample_count(&self) -> Option<i64> {
        self.get_i64_property(crate::properties::SAMPLES)
    }

    /// Get description property.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(channel) = file.get_channel("Group", "Channel") {
    ///     if let Some(desc) = channel.description() {
    ///         println!("Channel description: {}", desc);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn description(&self) -> Option<&str> {
        self.get_string_property(crate::properties::DESCRIPTION)
    }

    /// Get sensor type property.
    pub fn sensor_type(&self) -> Option<&str> {
        self.get_string_property(crate::properties::SENSOR_TYPE)
    }

    /// Get an i64 property value by name.
    pub fn get_i64_property(&self, name: &str) -> Option<i64> {
        match self.properties.get(name) {
            Some(PropertyValue::I64(i)) => Some(*i),
            Some(PropertyValue::I32(i)) => Some(*i as i64),
            Some(PropertyValue::I16(i)) => Some(*i as i64),
            Some(PropertyValue::I8(i)) => Some(*i as i64),
            _ => None,
        }
    }

    /// Convert TDMS timestamps to seconds since 1904 as f64.
    ///
    /// TDMS timestamps are stored as (seconds, fraction) where the fraction
    /// represents sub-second precision in units of 2^-64 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(channel) = file.get_channel("Time", "Timestamps") {
    ///     if let Some(times) = channel.as_timestamps_f64() {
    ///         for time in times {
    ///             println!("Time: {:.9} seconds since 1904", time);
    ///         }
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_timestamps_f64(&self) -> Option<Vec<f64>> {
        self.as_timestamps().map(|timestamps| {
            timestamps
                .iter()
                .map(|(seconds, fraction)| {
                    *seconds as f64 + (*fraction as f64 / (1u64 << 63) as f64) / 2.0
                })
                .collect()
        })
    }

    /// Convert TDMS timestamps to Unix epoch (seconds since 1970-01-01).
    ///
    /// This converts from TDMS epoch (1904-01-01) to Unix epoch (1970-01-01).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(channel) = file.get_channel("Time", "Timestamps") {
    ///     if let Some(unix_times) = channel.timestamps_to_unix() {
    ///         for time in unix_times {
    ///             println!("Unix time: {:.9}", time);
    ///         }
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn timestamps_to_unix(&self) -> Option<Vec<f64>> {
        const TDMS_TO_UNIX_OFFSET: i64 = 2082844800; // Seconds from 1904 to 1970

        self.as_timestamps_f64().map(|timestamps| {
            timestamps
                .iter()
                .map(|&t| t - TDMS_TO_UNIX_OFFSET as f64)
                .collect()
        })
    }
}

impl Default for TdmsFile {
    fn default() -> Self {
        Self::new()
    }
}

impl TdmsFile {
    /// Create a new empty TDMS file.
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
            groups: IndexMap::new(),
        }
    }

    /// Get an iterator over groups in this file.
    pub fn iter_groups(&self) -> impl Iterator<Item = (&str, &TdmsGroup)> {
        self.groups.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get a group by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(group) = file.group("Sensors") {
    ///     println!("Found group with {} channels", group.channels.len());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn group(&self, name: &str) -> Option<&TdmsGroup> {
        self.groups.get(name)
    }

    /// Get a channel directly by group and channel name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(channel) = file.get_channel("Sensors", "Temperature") {
    ///     println!("Found temperature channel with {} samples", channel.data_len());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_channel(&self, group: &str, channel: &str) -> Option<&TdmsChannel> {
        self.groups.get(group)?.channels.get(channel)
    }

    /// Try to get a channel, returning a descriptive error if not found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// match file.try_get_channel("Sensors", "Temperature") {
    ///     Ok(channel) => println!("Found channel with {} samples", channel.data_len()),
    ///     Err(e) => eprintln!("Channel not found: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_get_channel(&self, group: &str, channel: &str) -> Result<&TdmsChannel> {
        let group_obj = self
            .groups
            .get(group)
            .ok_or_else(|| crate::error::TdmsError::GroupNotFound(group.to_string()))?;

        group_obj.channels.get(channel).ok_or_else(|| {
            crate::error::TdmsError::ChannelNotFound(channel.to_string(), group.to_string())
        })
    }

    /// Load a TDMS file from the specified path.
    ///
    /// This method reads and parses the entire TDMS file, extracting all groups,
    /// channels, properties, and data. The file is read sequentially, handling
    /// multiple segments if present.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TDMS file to load
    ///
    /// # Returns
    ///
    /// Returns a `TdmsFile` containing all parsed data, or an error if the file
    /// cannot be read or parsed.
    ///
    /// # Errors
    ///
    /// This method can return errors for various reasons:
    /// - File not found or permission denied
    /// - Invalid TDMS file format or corrupted data
    /// - Unsupported TDMS features or data types
    /// - I/O errors during reading
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// // Load a TDMS file
    /// let file = TdmsFile::load(Path::new("measurements.tdms"))?;
    /// println!("Loaded {} groups", file.groups.len());
    ///
    /// // Handle errors gracefully
    /// match TdmsFile::load(Path::new("missing.tdms")) {
    ///     Ok(file) => println!("File loaded successfully"),
    ///     Err(e) => eprintln!("Failed to load file: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = TdmsReader::new(BufReader::new(file));

        let mut groups = IndexMap::new();
        let mut file_properties = IndexMap::new();

        loop {
            let segment = match reader.read_segment() {
                Ok(s) => s,
                Err(crate::error::TdmsError::Io(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break
                }
                Err(e) => return Err(e),
            };

            for obj in segment.objects {
                // If path has 0 components -> Root
                // If path has 1 component -> Group
                // If path has 2 components -> Channel

                if let Some(g_name) = obj.path.group_name() {
                    // Ensure group exists
                    let group = groups.entry(g_name.to_string()).or_insert(TdmsGroup {
                        properties: IndexMap::new(),
                        channels: IndexMap::new(),
                    });

                    if let Some(c_name) = obj.path.channel_name() {
                        // Add channel
                        let channel =
                            group
                                .channels
                                .entry(c_name.to_string())
                                .or_insert(TdmsChannel {
                                    properties: IndexMap::new(),
                                    data: None,
                                });
                        channel.properties.extend(obj.properties);
                        if let Some(new_data) = obj.data {
                            if let Some(existing_data) = &mut channel.data {
                                existing_data.extend(new_data)?;
                            } else {
                                channel.data = Some(new_data);
                            }
                        }
                    } else {
                        // Group properties
                        group.properties.extend(obj.properties);
                    }
                } else if obj.path.is_root() {
                    // File-level properties
                    file_properties.extend(obj.properties);
                }
            }
        }

        // Return populated groups
        Ok(Self {
            properties: file_properties,
            groups,
        })
    }
}
