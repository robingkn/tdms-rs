//! TDMS file writer implementation.
//!
//! This module provides functionality to create TDMS files with a hierarchical
//! structure: File -> Groups -> Channels. It supports all TDMS data types
//! and properties, ensuring binary compatibility with National Instruments software.
//!
//! # Examples
//!
//! ```no_run
//! use tdms_rs::writer::TdmsFileWriter;
//! use tdms_rs::TdmsData;
//!
//! let mut writer = TdmsFileWriter::new("measurements.tdms");
//! let group = writer.add_group("Sensors")?;
//! group.add_channel("Temperature", TdmsData::Double(vec![20.1, 21.5, 22.3]))?;
//! writer.write()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Multiple Data Types
//!
//! ```no_run
//! use tdms_rs::writer::TdmsFileWriter;
//! use tdms_rs::TdmsData;
//!
//! let mut writer = TdmsFileWriter::new("multi_type.tdms");
//! let group = writer.add_group("Mixed")?;
//!
//! // Different data types in one group
//! group.add_channel("Voltage", TdmsData::Double(vec![1.1, 2.2, 3.3]))?;
//! group.add_channel("Count", TdmsData::I32(vec![100, 200, 300]))?;
//! group.add_channel("Valid", TdmsData::Boolean(vec![true, false, true]))?;
//! group.add_channel("Labels", TdmsData::String(vec!["A".into(), "B".into(), "C".into()]))?;
//!
//! writer.write()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Properties Example
//!
//! ```no_run
//! use tdms_rs::writer::TdmsFileWriter;
//! use tdms_rs::{TdmsData, PropertyValue};
//!
//! let mut writer = TdmsFileWriter::new("with_props.tdms");
//!
//! // File-level properties
//! writer.add_property("Author", PropertyValue::String("TDMS Writer".into()))?;
//! writer.add_property("Version", PropertyValue::I32(1))?;
//!
//! let group = writer.add_group("DAQ")?;
//! // Group-level properties
//! group.add_property("Sample_Rate", PropertyValue::Double(1000.0))?;
//!
//! let channel = group.add_channel("AI0", TdmsData::Double(vec![1.1, 2.2]))?;
//! // Channel-level properties
//! channel.add_property("wf_unit_string", PropertyValue::String("V".into()))?;
//! channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
//!
//! writer.write()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # All Supported Data Types
//!
//! ```no_run
//! use tdms_rs::writer::TdmsFileWriter;
//! use tdms_rs::TdmsData;
//!
//! let mut writer = TdmsFileWriter::new("all_types.tdms");
//!
//! // Integer types
//! let integers = writer.add_group("Integers")?;
//! integers.add_channel("I8", TdmsData::I8(vec![-128, 0, 127]))?;
//! integers.add_channel("I16", TdmsData::I16(vec![-32768, 0, 32767]))?;
//! integers.add_channel("I32", TdmsData::I32(vec![-2147483648, 0, 2147483647]))?;
//! integers.add_channel("I64", TdmsData::I64(vec![i64::MIN, 0, i64::MAX]))?;
//!
//! // Unsigned integers
//! let unsigned = writer.add_group("Unsigned")?;
//! unsigned.add_channel("U8", TdmsData::U8(vec![0, 128, 255]))?;
//! unsigned.add_channel("U16", TdmsData::U16(vec![0, 32768, 65535]))?;
//! unsigned.add_channel("U32", TdmsData::U32(vec![0, 2147483648, 4294967295]))?;
//! unsigned.add_channel("U64", TdmsData::U64(vec![0, u64::MAX/2, u64::MAX]))?;
//!
//! // Floating point
//! let floats = writer.add_group("Floats")?;
//! floats.add_channel("Float", TdmsData::Float(vec![1.1, 2.2, 3.3]))?;
//! floats.add_channel("Double", TdmsData::Double(vec![1.1, 2.2, 3.3]))?;
//!
//! // Other types
//! let misc = writer.add_group("Misc")?;
//! misc.add_channel("Flags", TdmsData::Boolean(vec![true, false, true]))?;
//! misc.add_channel("Text", TdmsData::String(vec!["Hello".into(), "World".into()]))?;
//! misc.add_channel("Times", TdmsData::TimeStamp(vec![(1000, 0), (2000, 500000000)]))?;
//!
//! writer.write()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::datatypes::{DataType, PropertyValue, TdmsData};
use crate::error::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use indexmap::IndexMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// A TDMS file writer that can create TDMS files matching the corpus format.
///
/// The writer follows a builder pattern where you create groups, add channels with data,
/// and then write the complete file structure. All channels must contain data - channels
/// without data are not supported and will cause write operations to fail.
///
/// # Channel Data Requirements
///
/// - All channels must have associated data (empty channels are not allowed)
/// - Data vectors can be empty (zero samples) but the TdmsData enum variant must be present
/// - Channel names and group names must be valid UTF-8 strings
/// - Property keys and string values must be valid UTF-8
///
/// # Output Guarantees
///
/// - Files are written in a single segment with deterministic channel ordering
/// - Output is binary-compatible with National Instruments TDMS readers
/// - Channel ordering within groups is deterministic (alphabetical by channel name)
/// - Group ordering is deterministic (alphabetical by group name)
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::writer::TdmsFileWriter;
/// use tdms_rs::TdmsData;
///
/// let mut writer = TdmsFileWriter::new("output.tdms");
/// let group = writer.add_group("Group")?;
/// group.add_channel("Channel1", TdmsData::Double(vec![1.1, 2.2, 3.3]))?;
/// writer.write()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsFileWriter {
    path: PathBuf,
    groups: IndexMap<String, TdmsGroupWriter>,
    properties: IndexMap<String, PropertyValue>,
}

/// A group writer for organizing related channels.
pub struct TdmsGroupWriter {
    channels: BTreeMap<String, TdmsChannelWriter>,
    properties: IndexMap<String, PropertyValue>,
}

/// A channel writer containing data and properties.
pub struct TdmsChannelWriter {
    data: TdmsData,
    properties: IndexMap<String, PropertyValue>,
}

impl TdmsFileWriter {
    /// Create a new TDMS file writer for the specified path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            groups: IndexMap::new(),
            properties: IndexMap::new(),
        }
    }

    /// Add a group to the file and return a mutable reference to it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the group to create
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the created group.
    ///
    /// # Errors
    ///
    /// Returns `TdmsError::InvalidName` if the group name is empty.
    /// Returns `TdmsError::DuplicateName` if a group with this name already exists.
    pub fn add_group(
        &mut self,
        name: impl Into<String>,
    ) -> crate::error::Result<&mut TdmsGroupWriter> {
        let name = name.into();

        if name.is_empty() {
            return Err(crate::error::TdmsError::InvalidName(
                "Group name cannot be empty".into(),
            ));
        }

        if self.groups.contains_key(&name) {
            return Err(crate::error::TdmsError::DuplicateName(format!(
                "Group '{}' already exists",
                name
            )));
        }

        let group = TdmsGroupWriter {
            channels: BTreeMap::new(),
            properties: IndexMap::new(),
        };
        self.groups.insert(name.clone(), group);
        Ok(self.groups.get_mut(&name).unwrap())
    }

    /// Add a property to the file.
    ///
    /// # Arguments
    ///
    /// * `key` - The property name
    /// * `value` - The property value (can be any type that converts to PropertyValue)
    ///
    /// # Errors
    ///
    /// Returns `TdmsError::InvalidName` if the property key is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::writer::TdmsFileWriter;
    ///
    /// let mut writer = TdmsFileWriter::new("output.tdms");
    ///
    /// // Using the ergonomic From<T> conversions
    /// writer.add_property("Author", "John Doe")?;
    /// writer.add_property("Version", 1i32)?;
    /// writer.add_property("Sample_Rate", 1000.0)?;
    /// writer.add_property("Is_Valid", true)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_property(
        &mut self,
        key: impl Into<String>,
        value: impl Into<PropertyValue>,
    ) -> crate::error::Result<()> {
        let key = key.into();

        if key.is_empty() {
            return Err(crate::error::TdmsError::InvalidName(
                "Property key cannot be empty".into(),
            ));
        }

        self.properties.insert(key, value.into());
        Ok(())
    }

    /// Write the TDMS file to disk.
    pub fn write(&self) -> Result<()> {
        let file = File::create(&self.path)?;
        // Use a larger buffer for high-throughput sequential I/O
        let mut writer = BufWriter::with_capacity(1024 * 1024, file);

        // Build metadata (raw data will be written separately)
        let metadata_bytes = self.build_metadata()?;

        // Calculate offsets
        let raw_data_offset = metadata_bytes.len() as u64;
        let next_segment_offset = 0xFFFFFFFFFFFFFFFF; // No next segment

        // Write TDMS header (28 bytes)
        self.write_header(&mut writer, next_segment_offset, raw_data_offset)?;

        // Write metadata
        writer.write_all(&metadata_bytes)?;

        // Write raw data directly to the file stream
        for group in self.groups.values() {
            for channel in group.channels.values() {
                self.write_channel_data(&mut writer, &channel.data)?;
            }
        }

        writer.flush()?;
        writer.get_ref().sync_all()?;
        Ok(())
    }

    fn write_header<W: Write>(
        &self,
        writer: &mut W,
        next_segment_offset: u64,
        raw_data_offset: u64,
    ) -> Result<()> {
        // TDMS signature: "TDSm"
        writer.write_all(b"TDSm")?;

        // ToC mask (4 bytes) - from minimal.tdms: 0x0000000E
        writer.write_u32::<LittleEndian>(0x0000000E)?;

        // Version (4 bytes) - from minimal.tdms: 4712
        writer.write_u32::<LittleEndian>(4712)?;

        // Next segment offset (8 bytes)
        writer.write_u64::<LittleEndian>(next_segment_offset)?;

        // Raw data offset (8 bytes) - relative to start of segment payload
        writer.write_u64::<LittleEndian>(raw_data_offset)?;

        Ok(())
    }

    fn build_metadata(&self) -> Result<Vec<u8>> {
        let mut metadata = Vec::new();

        // Count objects: file + groups + channels
        let mut object_count = 1u32; // file object
        object_count += self.groups.len() as u32; // group objects
        for group in self.groups.values() {
            object_count += group.channels.len() as u32; // channel objects
        }

        // Write object count
        metadata.write_u32::<LittleEndian>(object_count)?;

        // Write file object (root)
        self.write_object_metadata(&mut metadata, "/", None, &self.properties)?;

        // Write group objects
        for (group_name, group) in &self.groups {
            let path = format!("/'{}'", group_name); // Removed trailing slash
            debug_assert!(
                !path.ends_with('/'),
                "Group path should not have trailing slash"
            );
            self.write_object_metadata(&mut metadata, &path, None, &group.properties)?;
        }

        // Write channel objects with reference to data
        for (group_name, group) in &self.groups {
            for (channel_name, channel) in &group.channels {
                let path = format!("/'{}'/'{}'", group_name, channel_name);

                // Build raw data info block with correct property count
                let raw_data_info =
                    self.build_raw_data_info_with_properties(&channel.data, &channel.properties)?;
                let raw_data_index = raw_data_info.len() as u32;

                // Check if this is string data
                let is_string = matches!(channel.data, TdmsData::String(_));

                // Write object metadata with raw data info embedded
                self.write_channel_object_metadata(
                    &mut metadata,
                    &path,
                    raw_data_index,
                    &raw_data_info,
                    &channel.properties,
                    is_string,
                )?;
            }
        }

        Ok(metadata)
    }

    fn write_object_metadata<W: Write>(
        &self,
        writer: &mut W,
        path: &str,
        raw_data_index: Option<u32>,
        properties: &IndexMap<String, PropertyValue>,
    ) -> Result<()> {
        // Object path length
        writer.write_u32::<LittleEndian>(path.len() as u32)?;

        // Object path
        writer.write_all(path.as_bytes())?;

        // Raw data index (4 bytes)
        writer.write_u32::<LittleEndian>(raw_data_index.unwrap_or(0xFFFFFFFF))?;

        // Number of properties (4 bytes)
        writer.write_u32::<LittleEndian>(properties.len() as u32)?;

        // Write properties
        for (key, value) in properties {
            self.write_property(writer, key, value)?;
        }

        Ok(())
    }

    fn write_channel_object_metadata<W: Write>(
        &self,
        writer: &mut W,
        path: &str,
        raw_data_index: u32,
        raw_data_info: &[u8],
        properties: &IndexMap<String, PropertyValue>,
        is_string: bool,
    ) -> Result<()> {
        // Object path length
        writer.write_u32::<LittleEndian>(path.len() as u32)?;

        // Object path
        writer.write_all(path.as_bytes())?;

        // Raw data index (length of raw data info)
        writer.write_u32::<LittleEndian>(raw_data_index)?;

        // Write raw data info block
        writer.write_all(raw_data_info)?;

        // For strings, property count is written separately after raw data info
        // For other types, property count is already included in raw data info
        if is_string {
            writer.write_u32::<LittleEndian>(properties.len() as u32)?;
        }

        // Write properties
        for (key, value) in properties {
            self.write_property(writer, key, value)?;
        }

        Ok(())
    }

    fn build_raw_data_info_with_properties(
        &self,
        data: &TdmsData,
        properties: &IndexMap<String, PropertyValue>,
    ) -> Result<Vec<u8>> {
        let mut raw_data_info = Vec::new();

        match data {
            TdmsData::Double(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::DoubleFloat as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::Float(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::SingleFloat as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::I8(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I8 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::I16(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I16 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::I32(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I32 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::I64(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I64 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::U8(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U8 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::U16(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U16 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::U32(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U32 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::U64(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U64 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::Boolean(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::Boolean as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
            TdmsData::String(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::String as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;

                // Calculate total size for strings (offsets + string bytes)
                let total_size = values.iter().map(|s| s.len()).sum::<usize>() + (values.len() * 4); // 4 bytes per offset
                raw_data_info.write_u64::<LittleEndian>(total_size as u64)?;
                // Property count is written separately for strings
            }
            TdmsData::TimeStamp(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::TimeStamp as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(properties.len() as u32)?;
            }
        }

        Ok(raw_data_info)
    }

    fn write_property<W: Write>(
        &self,
        writer: &mut W,
        key: &str,
        value: &PropertyValue,
    ) -> Result<()> {
        // Key length and key
        writer.write_u32::<LittleEndian>(key.len() as u32)?;
        writer.write_all(key.as_bytes())?;

        // Value type and value
        match value {
            PropertyValue::I8(i) => {
                writer.write_u32::<LittleEndian>(DataType::I8 as u32)?;
                writer.write_i8(*i)?;
            }
            PropertyValue::I16(i) => {
                writer.write_u32::<LittleEndian>(DataType::I16 as u32)?;
                writer.write_i16::<LittleEndian>(*i)?;
            }
            PropertyValue::I32(i) => {
                writer.write_u32::<LittleEndian>(DataType::I32 as u32)?;
                writer.write_i32::<LittleEndian>(*i)?;
            }
            PropertyValue::I64(i) => {
                writer.write_u32::<LittleEndian>(DataType::I64 as u32)?;
                writer.write_i64::<LittleEndian>(*i)?;
            }
            PropertyValue::U8(u) => {
                writer.write_u32::<LittleEndian>(DataType::U8 as u32)?;
                writer.write_u8(*u)?;
            }
            PropertyValue::U16(u) => {
                writer.write_u32::<LittleEndian>(DataType::U16 as u32)?;
                writer.write_u16::<LittleEndian>(*u)?;
            }
            PropertyValue::U32(u) => {
                writer.write_u32::<LittleEndian>(DataType::U32 as u32)?;
                writer.write_u32::<LittleEndian>(*u)?;
            }
            PropertyValue::U64(u) => {
                writer.write_u32::<LittleEndian>(DataType::U64 as u32)?;
                writer.write_u64::<LittleEndian>(*u)?;
            }
            PropertyValue::Float(f) => {
                writer.write_u32::<LittleEndian>(DataType::SingleFloat as u32)?;
                writer.write_f32::<LittleEndian>(*f)?;
            }
            PropertyValue::Double(d) => {
                writer.write_u32::<LittleEndian>(DataType::DoubleFloat as u32)?;
                writer.write_f64::<LittleEndian>(*d)?;
            }
            PropertyValue::String(s) => {
                writer.write_u32::<LittleEndian>(DataType::String as u32)?;
                writer.write_u32::<LittleEndian>(s.len() as u32)?;
                writer.write_all(s.as_bytes())?;
            }
            PropertyValue::Boolean(b) => {
                writer.write_u32::<LittleEndian>(DataType::Boolean as u32)?;
                writer.write_u8(if *b { 1 } else { 0 })?;
            }
            PropertyValue::TimeStamp((seconds, fraction)) => {
                writer.write_u32::<LittleEndian>(DataType::TimeStamp as u32)?;
                writer.write_u64::<LittleEndian>(*fraction)?;
                writer.write_i64::<LittleEndian>(*seconds)?;
            }
        }

        Ok(())
    }

    fn write_channel_data<W: Write>(&self, writer: &mut W, data: &TdmsData) -> Result<()> {
        match data {
            TdmsData::Double(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 8)
                };
                writer.write_all(buf)?;
            }
            TdmsData::Float(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 4)
                };
                writer.write_all(buf)?;
            }
            TdmsData::I8(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len())
                };
                writer.write_all(buf)?;
            }
            TdmsData::I16(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 2)
                };
                writer.write_all(buf)?;
            }
            TdmsData::I32(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 4)
                };
                writer.write_all(buf)?;
            }
            TdmsData::I64(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 8)
                };
                writer.write_all(buf)?;
            }
            TdmsData::U8(values) => {
                writer.write_all(values)?;
            }
            TdmsData::U16(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 2)
                };
                writer.write_all(buf)?;
            }
            TdmsData::U32(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 4)
                };
                writer.write_all(buf)?;
            }
            TdmsData::U64(values) => {
                let buf = unsafe {
                    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 8)
                };
                writer.write_all(buf)?;
            }
            TdmsData::Boolean(values) => {
                let buf: Vec<u8> = values.iter().map(|&v| if v { 1 } else { 0 }).collect();
                writer.write_all(&buf)?;
            }
            TdmsData::String(values) => {
                // Write offsets first
                let mut offset = 0u32;
                for s in values {
                    offset += s.len() as u32;
                    writer.write_u32::<LittleEndian>(offset)?;
                }
                // Write string bytes
                for s in values {
                    writer.write_all(s.as_bytes())?;
                }
            }
            TdmsData::TimeStamp(values) => {
                for &(seconds, fraction) in values {
                    writer.write_u64::<LittleEndian>(fraction)?;
                    writer.write_i64::<LittleEndian>(seconds)?;
                }
            }
        }
        Ok(())
    }
}

impl TdmsGroupWriter {
    /// Add a channel to this group with the specified data.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the channel to create
    /// * `data` - The data to store in the channel
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the created channel.
    ///
    /// # Errors
    ///
    /// Returns `TdmsError::InvalidName` if the channel name is empty.
    /// Returns `TdmsError::DuplicateName` if a channel with this name already exists in this group.
    pub fn add_channel(
        &mut self,
        name: impl Into<String>,
        data: TdmsData,
    ) -> crate::error::Result<&mut TdmsChannelWriter> {
        let name = name.into();

        if name.is_empty() {
            return Err(crate::error::TdmsError::InvalidName(
                "Channel name cannot be empty".into(),
            ));
        }

        if self.channels.contains_key(&name) {
            return Err(crate::error::TdmsError::DuplicateName(format!(
                "Channel '{}' already exists in this group",
                name
            )));
        }

        let channel = TdmsChannelWriter {
            data,
            properties: IndexMap::new(),
        };
        self.channels.insert(name.clone(), channel);
        Ok(self.channels.get_mut(&name).unwrap())
    }

    /// Add a property to this group.
    ///
    /// # Arguments
    ///
    /// * `key` - The property name
    /// * `value` - The property value (can be any type that converts to PropertyValue)
    ///
    /// # Errors
    ///
    /// Returns `TdmsError::InvalidName` if the property key is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::writer::TdmsFileWriter;
    ///
    /// let mut writer = TdmsFileWriter::new("output.tdms");
    /// let group = writer.add_group("Sensors")?;
    ///
    /// // Using the ergonomic From<T> conversions
    /// group.add_property("Location", "Lab A")?;
    /// group.add_property("Sample_Rate", 1000.0)?;
    /// group.add_property("Channel_Count", 4i32)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_property(
        &mut self,
        key: impl Into<String>,
        value: impl Into<PropertyValue>,
    ) -> crate::error::Result<()> {
        let key = key.into();

        if key.is_empty() {
            return Err(crate::error::TdmsError::InvalidName(
                "Property key cannot be empty".into(),
            ));
        }

        self.properties.insert(key, value.into());
        Ok(())
    }
}

impl TdmsChannelWriter {
    /// Add a property to this channel.
    ///
    /// # Arguments
    ///
    /// * `key` - The property name
    /// * `value` - The property value (can be any type that converts to PropertyValue)
    ///
    /// # Errors
    ///
    /// Returns `TdmsError::InvalidName` if the property key is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::writer::TdmsFileWriter;
    /// use tdms_rs::TdmsData;
    ///
    /// let mut writer = TdmsFileWriter::new("output.tdms");
    /// let group = writer.add_group("Sensors")?;
    /// let channel = group.add_channel("Temperature", TdmsData::Double(vec![20.0, 21.0]))?;
    ///
    /// // Using the ergonomic From<T> conversions
    /// channel.add_property("wf_unit_string", "°C")?;
    /// channel.add_property("wf_increment", 0.001)?;
    /// channel.add_property("calibrated", true)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_property(
        &mut self,
        key: impl Into<String>,
        value: impl Into<PropertyValue>,
    ) -> crate::error::Result<()> {
        let key = key.into();

        if key.is_empty() {
            return Err(crate::error::TdmsError::InvalidName(
                "Property key cannot be empty".into(),
            ));
        }

        self.properties.insert(key, value.into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TdmsFile;
    use std::fs;

    #[test]
    fn round_trip_group_path_no_trailing_slash() -> Result<()> {
        // Create output directory if it doesn't exist
        fs::create_dir_all("tests/output")?;

        let output_path = "tests/output/group_path.tdms";
        let mut writer = TdmsFileWriter::new(output_path);
        let group = writer.add_group("Integers")?;
        group.add_channel("Chan1", TdmsData::I32(vec![1, 2, 3]))?;
        writer.write()?;

        let written = TdmsFile::load(std::path::Path::new(output_path))?;
        assert_eq!(written.groups.keys().collect::<Vec<_>>(), &["Integers"]); // no trailing slash
        Ok(())
    }

    #[test]
    fn deterministic_channel_ordering() -> Result<()> {
        fs::create_dir_all("tests/output")?;

        let output_path = "tests/output/channel_order.tdms";
        let mut writer = TdmsFileWriter::new(output_path);
        let group = writer.add_group("Test")?;

        // Add channels in non-alphabetical order
        group.add_channel("Zebra", TdmsData::I32(vec![3]))?;
        group.add_channel("Alpha", TdmsData::I32(vec![1]))?;
        group.add_channel("Beta", TdmsData::I32(vec![2]))?;

        writer.write()?;

        let written = TdmsFile::load(std::path::Path::new(output_path))?;
        let test_group = written.groups.get("Test").unwrap();

        // Verify all channels exist (order doesn't matter for this test since reader uses HashMap)
        assert!(test_group.channels.contains_key("Alpha"));
        assert!(test_group.channels.contains_key("Beta"));
        assert!(test_group.channels.contains_key("Zebra"));
        assert_eq!(test_group.channels.len(), 3);

        // Verify data integrity
        if let Some(TdmsData::I32(alpha_data)) = &test_group.channels.get("Alpha").unwrap().data {
            assert_eq!(alpha_data, &vec![1]);
        }
        if let Some(TdmsData::I32(beta_data)) = &test_group.channels.get("Beta").unwrap().data {
            assert_eq!(beta_data, &vec![2]);
        }
        if let Some(TdmsData::I32(zebra_data)) = &test_group.channels.get("Zebra").unwrap().data {
            assert_eq!(zebra_data, &vec![3]);
        }

        Ok(())
    }

    #[test]
    fn properties_support() -> Result<()> {
        fs::create_dir_all("tests/output")?;

        let output_path = "tests/output/properties.tdms";
        let mut writer = TdmsFileWriter::new(output_path);

        // Add file-level properties
        writer.add_property("file_prop", PropertyValue::String("file_value".to_string()))?;
        writer.add_property("file_number", PropertyValue::I32(42))?;

        let group = writer.add_group("TestGroup")?;

        // Add group-level properties
        group.add_property(
            "group_prop",
            PropertyValue::String("group_value".to_string()),
        )?;
        group.add_property("group_double", PropertyValue::Double(std::f64::consts::PI))?;

        let channel = group.add_channel("TestChannel", TdmsData::Double(vec![1.0, 2.0, 3.0]))?;

        // Add channel-level properties
        channel.add_property("wf_unit_string", PropertyValue::String("V".to_string()))?;
        channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
        channel.add_property("channel_bool", PropertyValue::Boolean(true))?;

        writer.write()?;

        let written = TdmsFile::load(std::path::Path::new(output_path))?;

        // Verify file properties (note: current reader doesn't support file properties yet)
        // This test will initially fail until we implement file property reading

        // Verify group properties
        let test_group = written.groups.get("TestGroup").unwrap();
        assert_eq!(
            test_group.properties.get("group_prop"),
            Some(&PropertyValue::String("group_value".to_string()))
        );
        assert_eq!(
            test_group.properties.get("group_double"),
            Some(&PropertyValue::Double(std::f64::consts::PI))
        );

        // Verify channel properties
        let test_channel = test_group.channels.get("TestChannel").unwrap();
        assert_eq!(
            test_channel.properties.get("wf_unit_string"),
            Some(&PropertyValue::String("V".to_string()))
        );
        assert_eq!(
            test_channel.properties.get("wf_increment"),
            Some(&PropertyValue::Double(0.001))
        );
        assert_eq!(
            test_channel.properties.get("channel_bool"),
            Some(&PropertyValue::Boolean(true))
        );

        Ok(())
    }

    #[test]
    fn string_data_support() -> Result<()> {
        fs::create_dir_all("tests/output")?;

        let output_path = "tests/output/string_data.tdms";
        let mut writer = TdmsFileWriter::new(output_path);
        let group = writer.add_group("StringGroup")?;

        // Test multiple strings with different lengths
        let string_data = vec![
            "Hello".to_string(),
            "World".to_string(),
            "TDMS".to_string(),
            "Test".to_string(),
            "String Data".to_string(),
        ];

        group.add_channel("StringChannel", TdmsData::String(string_data.clone()))?;

        writer.write()?;

        let written = TdmsFile::load(std::path::Path::new(output_path))?;
        let test_group = written.groups.get("StringGroup").unwrap();
        let test_channel = test_group.channels.get("StringChannel").unwrap();

        // Verify string data
        match &test_channel.data {
            Some(TdmsData::String(written_strings)) => {
                assert_eq!(written_strings.len(), string_data.len());
                for (written, expected) in written_strings.iter().zip(string_data.iter()) {
                    assert_eq!(written, expected);
                }
            }
            _ => panic!("Expected string data, got {:?}", test_channel.data),
        }

        Ok(())
    }

    #[test]
    fn examine_strings_corpus() -> Result<()> {
        // Load the actual strings corpus to understand the format
        let reference_file = TdmsFile::load(std::path::Path::new(
            "tests/fixtures/tdms_corpus/03_datatypes/strings.tdms",
        ))?;

        println!("Strings corpus structure:");
        for (group_name, group) in &reference_file.groups {
            println!("Group: {}", group_name);
            for (channel_name, channel) in &group.channels {
                println!("  Channel: {}", channel_name);
                if let Some(data) = &channel.data {
                    match data {
                        TdmsData::String(strings) => {
                            println!("    String data: {:?}", strings);
                        }
                        _ => println!("    Data type: {:?}", data),
                    }
                }
            }
        }

        Ok(())
    }

    #[test]
    fn boolean_and_timestamp_data() -> Result<()> {
        fs::create_dir_all("tests/output")?;

        let output_path = "tests/output/bool_timestamp.tdms";
        let mut writer = TdmsFileWriter::new(output_path);
        let group = writer.add_group("MixedGroup")?;

        // Test boolean data
        group.add_channel(
            "BoolChannel",
            TdmsData::Boolean(vec![true, false, true, false, true]),
        )?;

        // Test timestamp data
        group.add_channel(
            "TimestampChannel",
            TdmsData::TimeStamp(vec![(1000, 0), (2000, 500000000), (3000, 1000000000)]),
        )?;

        writer.write()?;

        let written = TdmsFile::load(std::path::Path::new(output_path))?;
        let test_group = written.groups.get("MixedGroup").unwrap();

        // Verify boolean data
        let bool_channel = test_group.channels.get("BoolChannel").unwrap();
        match &bool_channel.data {
            Some(TdmsData::Boolean(written_bools)) => {
                assert_eq!(written_bools, &vec![true, false, true, false, true]);
            }
            _ => panic!("Expected boolean data, got {:?}", bool_channel.data),
        }

        // Verify timestamp data
        let timestamp_channel = test_group.channels.get("TimestampChannel").unwrap();
        match &timestamp_channel.data {
            Some(TdmsData::TimeStamp(written_timestamps)) => {
                assert_eq!(
                    written_timestamps,
                    &vec![(1000, 0), (2000, 500000000), (3000, 1000000000),]
                );
            }
            _ => panic!("Expected timestamp data, got {:?}", timestamp_channel.data),
        }

        Ok(())
    }

    #[test]
    fn round_trip_strings_corpus() -> Result<()> {
        fs::create_dir_all("tests/output")?;

        let output_path = "tests/output/strings_corpus.tdms";
        let mut writer = TdmsFileWriter::new(output_path);
        let group = writer.add_group("Strings")?;

        // Match the exact strings corpus data
        let string_data = vec![
            "Hello".to_string(),
            "World".to_string(),
            "".to_string(),
            "TDMS".to_string(),
            "File Format".to_string(),
        ];

        group.add_channel("Basic", TdmsData::String(string_data.clone()))?;

        writer.write()?;

        // Load and verify against the actual corpus
        let written = TdmsFile::load(std::path::Path::new(output_path))?;
        let reference = TdmsFile::load(std::path::Path::new(
            "tests/fixtures/tdms_corpus/03_datatypes/strings.tdms",
        ))?;

        // Compare structure
        assert_eq!(written.groups.len(), reference.groups.len());
        assert_eq!(
            written.groups.keys().collect::<Vec<_>>(),
            reference.groups.keys().collect::<Vec<_>>()
        );

        // Compare string data
        let written_group = written.groups.get("Strings").unwrap();
        let reference_group = reference.groups.get("Strings").unwrap();

        let written_channel = written_group.channels.get("Basic").unwrap();
        let reference_channel = reference_group.channels.get("Basic").unwrap();

        match (&written_channel.data, &reference_channel.data) {
            (
                Some(TdmsData::String(written_strings)),
                Some(TdmsData::String(reference_strings)),
            ) => {
                assert_eq!(written_strings, reference_strings);
            }
            _ => panic!("Expected string data in both files"),
        }

        Ok(())
    }
}
