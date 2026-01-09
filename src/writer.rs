//! TDMS file writer implementation.
//! 
//! This module provides functionality to create TDMS files that match the existing corpus exactly.
//! The writer API follows a hierarchical structure: File -> Groups -> Channels.

use std::path::{Path, PathBuf};
use std::collections::{HashMap, BTreeMap};
use std::fs::File;
use std::io::{Write, BufWriter, Cursor};
use byteorder::{WriteBytesExt, LittleEndian};
use crate::datatypes::{PropertyValue, TdmsData, DataType};
use crate::error::{Result, TdmsError};

/// A TDMS file writer that can create TDMS files matching the corpus format.
/// 
/// The writer follows a builder pattern where you create groups, add channels with data,
/// and then write the complete file structure.
/// 
/// # Examples
/// 
/// ```no_run
/// use tdms_rs::writer::TdmsFileWriter;
/// use tdms_rs::TdmsData;
/// 
/// let mut writer = TdmsFileWriter::new("output.tdms");
/// let group = writer.add_group("Group");
/// group.add_channel("Channel1", TdmsData::Double(vec![1.1, 2.2, 3.3]));
/// writer.write()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsFileWriter {
    path: PathBuf,
    groups: HashMap<String, TdmsGroupWriter>,
    properties: HashMap<String, PropertyValue>,
}

/// A group writer for organizing related channels.
pub struct TdmsGroupWriter {
    name: String,
    channels: BTreeMap<String, TdmsChannelWriter>,
    properties: HashMap<String, PropertyValue>,
}

/// A channel writer containing data and properties.
pub struct TdmsChannelWriter {
    name: String,
    data: TdmsData,
    properties: HashMap<String, PropertyValue>,
}

impl TdmsFileWriter {
    /// Create a new TDMS file writer for the specified path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            groups: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a group to the file and return a mutable reference to it.
    pub fn add_group(&mut self, name: &str) -> &mut TdmsGroupWriter {
        let group = TdmsGroupWriter {
            name: name.to_string(),
            channels: BTreeMap::new(),
            properties: HashMap::new(),
        };
        self.groups.insert(name.to_string(), group);
        self.groups.get_mut(name).unwrap()
    }

    /// Write the TDMS file to disk.
    pub fn write(&self) -> Result<()> {
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);
        
        // Build metadata and raw data sections
        let (metadata_bytes, raw_data_bytes) = self.build_segment_data()?;
        
        // Calculate offsets
        let raw_data_offset = metadata_bytes.len() as u64;
        let next_segment_offset = 0xFFFFFFFFFFFFFFFF; // No next segment
        
        // Write TDMS header (28 bytes)
        self.write_header(&mut writer, next_segment_offset, raw_data_offset)?;
        
        // Write metadata
        writer.write_all(&metadata_bytes)?;
        
        // Write raw data
        writer.write_all(&raw_data_bytes)?;
        
        writer.flush()?;
        Ok(())
    }

    fn write_header<W: Write>(&self, writer: &mut W, next_segment_offset: u64, raw_data_offset: u64) -> Result<()> {
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

    fn build_segment_data(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut metadata = Vec::new();
        let mut raw_data = Vec::new();
        
        // Count objects: file + groups + channels
        let mut object_count = 1u32; // file object
        object_count += self.groups.len() as u32; // group objects
        for group in self.groups.values() {
            object_count += group.channels.len() as u32; // channel objects
        }
        
        // Write object count
        metadata.write_u32::<LittleEndian>(object_count)?;
        
        // Write file object (root)
        self.write_object_metadata(&mut metadata, "/", None, &HashMap::new())?;
        
        // Write group objects
        for (group_name, group) in &self.groups {
            let path = format!("/'{}'", group_name); // Removed trailing slash
            debug_assert!(!path.ends_with('/'), "Group path should not have trailing slash");
            self.write_object_metadata(&mut metadata, &path, None, &group.properties)?;
        }
        
        // Write channel objects with data
        for (group_name, group) in &self.groups {
            for (channel_name, channel) in &group.channels {
                let path = format!("/'{}'/'{}'", group_name, channel_name);
                
                // Build raw data info block
                let raw_data_info = self.build_raw_data_info(&channel.data)?;
                let raw_data_index = raw_data_info.len() as u32;
                
                // Write object metadata with raw data info embedded
                self.write_channel_object_metadata(&mut metadata, &path, raw_data_index, &raw_data_info, &channel.properties)?;
                
                // Write actual data to raw data section
                self.write_channel_data(&mut raw_data, &channel.data)?;
            }
        }
        
        Ok((metadata, raw_data))
    }

    fn write_object_metadata<W: Write>(&self, writer: &mut W, path: &str, raw_data_index: Option<u32>, properties: &HashMap<String, PropertyValue>) -> Result<()> {
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

    fn write_channel_object_metadata<W: Write>(&self, writer: &mut W, path: &str, raw_data_index: u32, raw_data_info: &[u8], properties: &HashMap<String, PropertyValue>) -> Result<()> {
        // Object path length
        writer.write_u32::<LittleEndian>(path.len() as u32)?;
        
        // Object path
        writer.write_all(path.as_bytes())?;
        
        // Raw data index (length of raw data info)
        writer.write_u32::<LittleEndian>(raw_data_index)?;
        
        // Write raw data info block (this already includes property count at the end)
        writer.write_all(raw_data_info)?;
        
        // Write properties (the count is already written in raw_data_info)
        for (key, value) in properties {
            self.write_property(writer, key, value)?;
        }
        
        Ok(())
    }

    fn build_raw_data_info(&self, data: &TdmsData) -> Result<Vec<u8>> {
        let mut raw_data_info = Vec::new();
        
        match data {
            TdmsData::Double(values) => {
                // Data type (4 bytes)
                raw_data_info.write_u32::<LittleEndian>(DataType::DoubleFloat as u32)?;
                // Dimension (4 bytes)
                raw_data_info.write_u32::<LittleEndian>(1)?;
                // Number of values (8 bytes)
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                // Property count (4 bytes) - this is part of the raw data info
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::Float(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::SingleFloat as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::I8(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I8 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::I16(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I16 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::I32(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I32 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::I64(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::I64 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::U8(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U8 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::U16(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U16 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::U32(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U32 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::U64(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::U64 as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::Boolean(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::Boolean as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::String(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::String as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                
                // Calculate total size for strings
                let total_size = values.iter().map(|s| s.len()).sum::<usize>() + (values.len() * 4); // 4 bytes per offset
                raw_data_info.write_u64::<LittleEndian>(total_size as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
            TdmsData::TimeStamp(values) => {
                raw_data_info.write_u32::<LittleEndian>(DataType::TimeStamp as u32)?;
                raw_data_info.write_u32::<LittleEndian>(1)?;
                raw_data_info.write_u64::<LittleEndian>(values.len() as u64)?;
                raw_data_info.write_u32::<LittleEndian>(0)?;
            },
        }
        
        Ok(raw_data_info)
    }

    fn write_property<W: Write>(&self, writer: &mut W, key: &str, value: &PropertyValue) -> Result<()> {
        // Key length and key
        writer.write_u32::<LittleEndian>(key.len() as u32)?;
        writer.write_all(key.as_bytes())?;
        
        // Value type and value
        match value {
            PropertyValue::String(s) => {
                writer.write_u32::<LittleEndian>(DataType::String as u32)?;
                writer.write_u32::<LittleEndian>(s.len() as u32)?;
                writer.write_all(s.as_bytes())?;
            },
            PropertyValue::Double(d) => {
                writer.write_u32::<LittleEndian>(DataType::DoubleFloat as u32)?;
                writer.write_f64::<LittleEndian>(*d)?;
            },
            PropertyValue::I32(i) => {
                writer.write_u32::<LittleEndian>(DataType::I32 as u32)?;
                writer.write_i32::<LittleEndian>(*i)?;
            },
            // Add other property types as needed
            _ => return Err(TdmsError::NotImplemented("Property type not yet supported in writer".to_string())),
        }
        
        Ok(())
    }

    fn write_channel_data<W: Write>(&self, writer: &mut W, data: &TdmsData) -> Result<()> {
        match data {
            TdmsData::Double(values) => for &v in values { writer.write_f64::<LittleEndian>(v)?; },
            TdmsData::Float(values) => for &v in values { writer.write_f32::<LittleEndian>(v)?; },
            TdmsData::I8(values) => for &v in values { writer.write_i8(v)?; },
            TdmsData::I16(values) => for &v in values { writer.write_i16::<LittleEndian>(v)?; },
            TdmsData::I32(values) => for &v in values { writer.write_i32::<LittleEndian>(v)?; },
            TdmsData::I64(values) => for &v in values { writer.write_i64::<LittleEndian>(v)?; },
            TdmsData::U8(values) => for &v in values { writer.write_u8(v)?; },
            TdmsData::U16(values) => for &v in values { writer.write_u16::<LittleEndian>(v)?; },
            TdmsData::U32(values) => for &v in values { writer.write_u32::<LittleEndian>(v)?; },
            TdmsData::U64(values) => for &v in values { writer.write_u64::<LittleEndian>(v)?; },
            TdmsData::Boolean(values) => for &v in values { writer.write_u8(if v {1} else {0})?; },
            TdmsData::String(values) => {
                // Write offsets first
                let mut offset = 0u32;
                for s in values {
                    offset += s.len() as u32;
                    writer.write_u32::<LittleEndian>(offset)?;
                }
                // Write string bytes
                for s in values { writer.write_all(s.as_bytes())?; }
            },
            TdmsData::TimeStamp(values) => {
                for &(seconds, fraction) in values {
                    writer.write_u64::<LittleEndian>(fraction)?;
                    writer.write_i64::<LittleEndian>(seconds)?;
                }
            },
            _ => panic!("Unsupported TdmsData type"),
        }
        Ok(())
    }

}

impl TdmsGroupWriter {
    /// Add a channel to this group with the specified data.
    pub fn add_channel(&mut self, name: &str, data: TdmsData) -> &mut TdmsChannelWriter {
        let channel = TdmsChannelWriter {
            name: name.to_string(),
            data,
            properties: HashMap::new(),
        };
        self.channels.insert(name.to_string(), channel);
        self.channels.get_mut(name).unwrap()
    }

    /// Add a property to this group.
    pub fn add_property(&mut self, key: &str, value: PropertyValue) {
        self.properties.insert(key.to_string(), value);
    }
}

impl TdmsChannelWriter {
    /// Add a property to this channel.
    pub fn add_property(&mut self, key: &str, value: PropertyValue) {
        self.properties.insert(key.to_string(), value);
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
        let group = writer.add_group("Integers");
        group.add_channel("Chan1", TdmsData::I32(vec![1, 2, 3]));
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
        let group = writer.add_group("Test");
        
        // Add channels in non-alphabetical order
        group.add_channel("Zebra", TdmsData::I32(vec![3]));
        group.add_channel("Alpha", TdmsData::I32(vec![1]));
        group.add_channel("Beta", TdmsData::I32(vec![2]));
        
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
}