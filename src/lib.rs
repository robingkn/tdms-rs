//! # tdms-rs
//!
//! A pure Rust library for reading and writing National Instruments TDMS
//! (Technical Data Management Streaming) files with full format support and high performance.
//!
//! ## Quick Start
//!
//! ### Reading TDMS Files
//!
//! ```no_run
//! use tdms_rs::TdmsFile;
//! use std::path::Path;
//!
//! let file = TdmsFile::load(Path::new("data.tdms"))?;
//!
//! if let Some(channel) = file.get_channel("Sensors", "Temperature") {
//!     let expected_count = channel.data_len();
//!     let mut buffer = vec![0.0f64; expected_count];
//!     if let Ok(count) = channel.read_f64_into(&mut buffer) {
//!         let avg = buffer.iter().take(count).sum::<f64>() / count as f64;
//!         println!("Average: {:.2}", avg);
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
//! let mut writer = TdmsFileWriter::new("output.tdms");
//! writer.add_property("Author", "Rust App")?;
//!
//! let group = writer.add_group("Sensors")?;
//! group.add_channel("Temperature", TdmsData::Double(vec![20.1, 21.5, 22.3]))?;
//!
//! writer.write()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Supported Data Types
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

pub mod datatypes;
pub mod error;
pub mod metadata;
pub mod reader;
pub mod segment;
pub mod writer;

use crate::error::{Result, TdmsError};
use crate::reader::TdmsReader;
use std::fs::File;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

    /// The path to the file on disk, used for lazy loading.
    pub(crate) _file_path: Option<PathBuf>,
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

    pub(crate) data_locations: Vec<crate::metadata::DataLocation>,
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) cache: OnceLock<TdmsData>,
    pub(crate) data_type: Option<crate::datatypes::DataType>,
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
        Self::new("")
    }
}

impl TdmsChannel {
    /// Create a new empty channel.
    pub fn new(name: impl Into<String>) -> Self {
        let _unused_name = name.into();
        Self {
            properties: IndexMap::new(),
            data: None,
            data_locations: Vec::new(),
            file_path: None,
            cache: OnceLock::new(),
            data_type: None,
        }
    }
    // All as_* methods have been removed in favor of explicit read_into methods.
    // Use read_f64_into(), read_i32_into(), etc. instead.
    // This change eliminates hidden allocations and makes I/O explicit.

    /// Get the number of data samples in this channel.
    pub fn data_len(&self) -> usize {
        if let Some(data) = &self.data {
            data.len()
        } else if let Some(data) = self.cache.get() {
            data.len()
        } else {
            self.data_locations
                .iter()
                .map(|loc| loc.number_of_values as usize)
                .sum()
        }
    }

    /// Get a human-readable name for the data type.
    pub fn data_type_name(&self) -> Option<&'static str> {
        if let Some(data) = &self.data {
            Some(data.type_name())
        } else if let Some(data) = self.cache.get() {
            Some(data.type_name())
        } else {
            self.data_locations.first().map(|loc| loc.data_type.type_name_static())
        }
    }

    /// Ensures the channel data is loaded from disk.
    /// This is called automatically by data accessors.
    pub fn ensure_data_loaded(&self) -> Result<&TdmsData> {
        if let Some(data) = &self.data {
            return Ok(data);
        }

        if let Some(data) = self.cache.get() {
            return Ok(data);
        }

        let path = self.file_path.as_ref().ok_or(TdmsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File path not set for lazy loading",
        )))?;

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut aggregated_data: Option<TdmsData> = None;

        for loc in &self.data_locations {
            reader.seek(std::io::SeekFrom::Start(loc.offset))?;
            let data = crate::datatypes::read_raw_data(
                &mut reader,
                &loc.data_type,
                loc.number_of_values,
                loc.total_size_bytes,
            )?;

            if let Some(existing) = &mut aggregated_data {
                existing.extend(data)?;
            } else {
                aggregated_data = Some(data);
            }
        }

        let data = if let Some(d) = aggregated_data {
            d
        } else {
            // No data locations or all were empty. Use data_type if available.
            let dt = self.data_type.as_ref().or_else(|| {
                self.data_locations.first().map(|loc| &loc.data_type)
            });
            
            if let Some(dt) = dt {
                crate::datatypes::create_empty_data(dt)?
            } else {
                return Err(TdmsError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "No data found for channel and data type unknown",
                )));
            }
        };

        // Try to initialize. If someone else beat us to it, that's fine.
        let _ = self.cache.set(data);

        // Return the reference from the cache
        Ok(self.cache.get().unwrap())
    }

    /// Read channel data directly into a caller-provided buffer.
    ///
    /// This method performs explicit I/O to read channel data from disk into
    /// the provided buffer. It supports multi-segment files by aggregating data
    /// from all segments. No allocations are performed for the data itself.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Caller-owned buffer to fill with data
    ///
    /// # Returns
    ///
    /// Number of elements read into the buffer. May be less than buffer length
    /// if the channel contains less data than the buffer size.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The channel data type doesn't match the buffer type
    /// - I/O errors occur during reading
    /// - File path is not set (channel wasn't loaded from file)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    ///
    /// let file = TdmsFile::load(Path::new("data.tdms"))?;
    /// if let Some(channel) = file.get_channel("Group", "Channel") {
    ///     let expected_count = channel.data_len();
    ///     let mut buffer = vec![0.0f64; expected_count];
    ///     let read_count = channel.read_f64_into(&mut buffer)?;
    ///     println!("Read {} f64 values", read_count);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read_f64_into(&self, buffer: &mut [f64]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::DoubleFloat,
            |reader, buf| crate::datatypes::read_f64_into(reader, buf),
        )
    }

    pub fn read_f32_into(&self, buffer: &mut [f32]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::SingleFloat,
            |reader, buf| crate::datatypes::read_f32_into(reader, buf),
        )
    }

    pub fn read_i8_into(&self, buffer: &mut [i8]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::I8,
            |reader, buf| crate::datatypes::read_i8_into(reader, buf),
        )
    }

    pub fn read_i16_into(&self, buffer: &mut [i16]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::I16,
            |reader, buf| crate::datatypes::read_i16_into(reader, buf),
        )
    }

    pub fn read_i32_into(&self, buffer: &mut [i32]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::I32,
            |reader, buf| crate::datatypes::read_i32_into(reader, buf),
        )
    }

    pub fn read_i64_into(&self, buffer: &mut [i64]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::I64,
            |reader, buf| crate::datatypes::read_i64_into(reader, buf),
        )
    }

    pub fn read_u8_into(&self, buffer: &mut [u8]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::U8,
            |reader, buf| crate::datatypes::read_u8_into(reader, buf),
        )
    }

    pub fn read_u16_into(&self, buffer: &mut [u16]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::U16,
            |reader, buf| crate::datatypes::read_u16_into(reader, buf),
        )
    }

    pub fn read_u32_into(&self, buffer: &mut [u32]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::U32,
            |reader, buf| crate::datatypes::read_u32_into(reader, buf),
        )
    }

    pub fn read_u64_into(&self, buffer: &mut [u64]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::U64,
            |reader, buf| crate::datatypes::read_u64_into(reader, buf),
        )
    }

    pub fn read_bool_into(&self, buffer: &mut [bool]) -> Result<usize> {
        self.read_numeric_into(
            buffer,
            crate::datatypes::DataType::Boolean,
            |reader, buf| crate::datatypes::read_bool_into(reader, buf),
        )
    }

    pub fn read_timestamp_into(&self, buffer: &mut [(i64, u64)]) -> Result<usize> {
        let path = self.file_path.as_ref().ok_or(TdmsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File path not set for lazy loading",
        )))?;

        if self.data_locations.is_empty() {
            return Ok(0);
        }

        // Check type matches
        let expected_type = self.data_type.as_ref().or_else(|| {
            self.data_locations.first().map(|loc| &loc.data_type)
        });
        
        match expected_type {
            Some(crate::datatypes::DataType::TimeStamp) => {},
            _ => {
                return Err(TdmsError::InvalidFormat(
                    format!("Channel data type is not TimeStamp, got {:?}", expected_type)
                ));
            }
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut offset = 0;

        for loc in &self.data_locations {
            if offset >= buffer.len() {
                break;
            }

            reader.seek(std::io::SeekFrom::Start(loc.offset))?;
            
            let segment_count = loc.number_of_values as usize;
            let to_read = (buffer.len() - offset).min(segment_count);
            
            if to_read == 0 {
                continue;
            }

            let segment_buffer = &mut buffer[offset..offset + to_read];
            let read_count = crate::datatypes::read_timestamp_into(&mut reader, segment_buffer)?;
            
            offset += read_count;
        }

        Ok(offset)
    }

    /// Internal helper for reading numeric types into slices.
    /// Handles multi-segment files by aggregating reads across all data locations.
    fn read_numeric_into<T>(
        &self,
        buffer: &mut [T],
        expected_type: crate::datatypes::DataType,
        read_fn: impl Fn(&mut BufReader<File>, &mut [T]) -> Result<usize>,
    ) -> Result<usize> {
        let path = self.file_path.as_ref().ok_or(TdmsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File path not set for lazy loading",
        )))?;

        if self.data_locations.is_empty() {
            return Ok(0);
        }

        // Check type matches
        let actual_type = self.data_type.as_ref().or_else(|| {
            self.data_locations.first().map(|loc| &loc.data_type)
        });
        
        match actual_type {
            Some(actual) if *actual == expected_type => {},
            Some(other) => {
                return Err(TdmsError::InvalidFormat(
                    format!("Type mismatch: expected {:?}, got {:?}", expected_type, other)
                ));
            }
            None => {
                return Err(TdmsError::InvalidFormat(
                    "Channel data type unknown".to_string()
                ));
            }
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut offset = 0;

        for loc in &self.data_locations {
            if offset >= buffer.len() {
                break;
            }

            // Verify type matches for this segment
            if loc.data_type != expected_type {
                return Err(TdmsError::InvalidFormat(
                    format!("Segment type mismatch: expected {:?}, got {:?}", expected_type, loc.data_type)
                ));
            }

            reader.seek(std::io::SeekFrom::Start(loc.offset))?;
            
            let segment_count = loc.number_of_values as usize;
            let to_read = (buffer.len() - offset).min(segment_count);
            
            if to_read == 0 {
                continue;
            }

            let segment_buffer = &mut buffer[offset..offset + to_read];
            let read_count = read_fn(&mut reader, segment_buffer)?;
            
            offset += read_count;
        }

        Ok(offset)
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

    // Timestamp conversion methods removed - they allocated memory.
    // Users should read timestamps with read_timestamp_into() and convert manually if needed.
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
            _file_path: None,
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
                                    data_locations: Vec::new(),
                                    file_path: Some(path.to_path_buf()),
                                    cache: OnceLock::new(),
                                    data_type: None,
                                });
                        channel.properties.extend(obj.properties);
                        if let Some(loc) = obj.data_location {
                            channel.data_locations.push(loc);
                        }
                        if let Some(meta) = obj.raw_data_meta {
                            channel.data_type = Some(meta.data_type);
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
            _file_path: Some(path.to_path_buf()),
        })
    }
}
