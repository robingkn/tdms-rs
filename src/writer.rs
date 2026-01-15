//! TDMS file writer implementation.

use crate::datatypes::DataType;
use crate::error::{Result, TdmsError};
use crate::PropertyValue;
use byteorder::{LittleEndian, WriteBytesExt};
use indexmap::IndexMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A TDMS file writer.
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::TdmsWriter;
///
/// let mut w = TdmsWriter::create("out.tdms")?;
/// let mut g = w.add_group("DAQ")?;
/// let mut ch = g.add_channel::<f64>("Voltage")?;
/// ch.write(&[1.0, 2.0, 3.0])?;
/// w.close()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsWriter {
    path: PathBuf,
    groups: IndexMap<String, WriterGroupData>,
    properties: IndexMap<String, PropertyValue>,
    closed: bool,
}

struct WriterGroupData {
    name: String,
    channels: BTreeMap<String, WriterChannelData>,
    properties: IndexMap<String, PropertyValue>,
}

struct WriterChannelData {
    name: String,
    data_type: DataType,
    data: Vec<u8>,
    properties: IndexMap<String, PropertyValue>,
}

/// A group within a TDMS writer.
pub struct WriterGroup<'w> {
    writer: &'w mut TdmsWriter,
    group_name: String,
}

/// A channel within a writer group.
pub struct WriterChannel<'w, T> {
    writer: &'w mut TdmsWriter,
    group_name: String,
    channel_name: String,
    _phantom: PhantomData<T>,
}

impl TdmsWriter {
    /// Create a new TDMS file for writing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsWriter;
    ///
    /// let mut w = TdmsWriter::create("output.tdms")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            groups: IndexMap::new(),
            properties: IndexMap::new(),
            closed: false,
        })
    }

    /// Add a group to the file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsWriter;
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// let mut g = w.add_group("Sensors")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_group(&mut self, name: impl Into<String>) -> Result<WriterGroup<'_>> {
        let name = name.into();

        if name.is_empty() {
            return Err(TdmsError::InvalidName("Group name cannot be empty".into()));
        }

        if !self.groups.contains_key(&name) {
            self.groups.insert(
                name.clone(),
                WriterGroupData {
                    name: name.clone(),
                    channels: BTreeMap::new(),
                    properties: IndexMap::new(),
                },
            );
        }

        Ok(WriterGroup {
            writer: self,
            group_name: name,
        })
    }

    /// Add a property to the file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::{TdmsWriter, PropertyValue};
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// w.add_property("Author", PropertyValue::String("John Doe".into()))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        value: PropertyValue,
    ) -> Result<&mut Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName("Property name cannot be empty".into()));
        }
        self.properties.insert(name, value);
        Ok(self)
    }

    /// Close and finalize the TDMS file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsWriter;
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// // ... add data ...
    /// w.close()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn close(mut self) -> Result<()> {
        if self.closed {
            return Err(TdmsError::WriterClosed);
        }

        self.write_file()?;
        self.closed = true;
        Ok(())
    }

    /// Abort writing and delete the partial file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsWriter;
    ///
    /// let mut w = TdmsWriter::create("partial.tdms")?;
    /// // ... something goes wrong ...
    /// w.abort()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn abort(self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    fn write_file(&mut self) -> Result<()> {
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);

        // Write lead-in
        writer.write_all(b"TDSm")?;

        // ToC mask
        let toc = 0x0E; // Has metadata, raw data, new object list
        writer.write_u32::<LittleEndian>(toc)?;

        // Version
        writer.write_u32::<LittleEndian>(4712)?;

        // Placeholder for next segment offset and raw data offset
        let segment_offset_pos = writer.stream_position()?;
        writer.write_u64::<LittleEndian>(0xFFFFFFFFFFFFFFFF)?; // Next segment
        writer.write_u64::<LittleEndian>(0)?; // Raw data offset (will update)

        // Write metadata
        let _metadata_start = writer.stream_position()?;

        // Count objects
        let mut object_count = 0;
        if !self.properties.is_empty() {
            object_count += 1; // Root object
        }
        for group in self.groups.values() {
            object_count += 1; // Always count group
            object_count += group.channels.len() as u32;
        }

        writer.write_u32::<LittleEndian>(object_count)?;

        // Write root object if it has properties
        if !self.properties.is_empty() {
            self.write_object(&mut writer, "/", &self.properties, None)?;
        }

        // Write groups and channels
        for group in self.groups.values() {
            let group_path = format!("/'{}'", group.name);

            // Always write group object (even without properties) so it exists
            self.write_object(&mut writer, &group_path, &group.properties, None)?;

            for channel in group.channels.values() {
                let channel_path = format!("/'{}'/'{}'", group.name, channel.name);
                self.write_object(
                    &mut writer,
                    &channel_path,
                    &channel.properties,
                    Some((&channel.data_type, channel.data.len())),
                )?;
            }
        }

        // Write raw data
        let raw_data_offset = writer.stream_position()?;

        for group in self.groups.values() {
            for channel in group.channels.values() {
                writer.write_all(&channel.data)?;
            }
        }

        // Update raw data offset
        let end_pos = writer.stream_position()?;
        writer.seek(std::io::SeekFrom::Start(segment_offset_pos + 8))?;
        writer.write_u64::<LittleEndian>(raw_data_offset - 28)?;
        writer.seek(std::io::SeekFrom::Start(end_pos))?;

        writer.flush()?;
        Ok(())
    }

    fn write_object(
        &self,
        writer: &mut BufWriter<File>,
        path: &str,
        properties: &IndexMap<String, PropertyValue>,
        raw_data: Option<(&DataType, usize)>,
    ) -> Result<()> {
        // Write path
        writer.write_u32::<LittleEndian>(path.len() as u32)?;
        writer.write_all(path.as_bytes())?;

        // Raw data index
        let raw_data_index = if raw_data.is_some() {
            20_u32 // Has raw data: Type(4) + Dim(4) + Count(8) + PropCount(4) = 20
        } else {
            0xFFFFFFFF_u32 // No raw data
        };
        writer.write_u32::<LittleEndian>(raw_data_index)?;

        // Write raw data metadata if present
        if let Some((dtype, byte_len)) = raw_data {
            writer.write_u32::<LittleEndian>(dtype.to_u32())?;
            writer.write_u32::<LittleEndian>(1)?; // Array dimension
            let count = byte_len / dtype.itemsize();
            writer.write_u64::<LittleEndian>(count as u64)?;
        }

        // Write properties
        writer.write_u32::<LittleEndian>(properties.len() as u32)?;
        for (key, value) in properties {
            writer.write_u32::<LittleEndian>(key.len() as u32)?;
            writer.write_all(key.as_bytes())?;

            self.write_property_value(writer, value)?;
        }

        Ok(())
    }

    fn write_property_value(
        &self,
        writer: &mut BufWriter<File>,
        value: &PropertyValue,
    ) -> Result<()> {
        match value {
            PropertyValue::I8(v) => {
                writer.write_u32::<LittleEndian>(DataType::I8.to_u32())?;
                writer.write_i8(*v)?;
            }
            PropertyValue::I16(v) => {
                writer.write_u32::<LittleEndian>(DataType::I16.to_u32())?;
                writer.write_i16::<LittleEndian>(*v)?;
            }
            PropertyValue::I32(v) => {
                writer.write_u32::<LittleEndian>(DataType::I32.to_u32())?;
                writer.write_i32::<LittleEndian>(*v)?;
            }
            PropertyValue::I64(v) => {
                writer.write_u32::<LittleEndian>(DataType::I64.to_u32())?;
                writer.write_i64::<LittleEndian>(*v)?;
            }
            PropertyValue::U8(v) => {
                writer.write_u32::<LittleEndian>(DataType::U8.to_u32())?;
                writer.write_u8(*v)?;
            }
            PropertyValue::U16(v) => {
                writer.write_u32::<LittleEndian>(DataType::U16.to_u32())?;
                writer.write_u16::<LittleEndian>(*v)?;
            }
            PropertyValue::U32(v) => {
                writer.write_u32::<LittleEndian>(DataType::U32.to_u32())?;
                writer.write_u32::<LittleEndian>(*v)?;
            }
            PropertyValue::U64(v) => {
                writer.write_u32::<LittleEndian>(DataType::U64.to_u32())?;
                writer.write_u64::<LittleEndian>(*v)?;
            }
            PropertyValue::Float(v) => {
                writer.write_u32::<LittleEndian>(DataType::SingleFloat.to_u32())?;
                writer.write_f32::<LittleEndian>(*v)?;
            }
            PropertyValue::Double(v) => {
                writer.write_u32::<LittleEndian>(DataType::DoubleFloat.to_u32())?;
                writer.write_f64::<LittleEndian>(*v)?;
            }
            PropertyValue::String(s) => {
                writer.write_u32::<LittleEndian>(DataType::String.to_u32())?;
                writer.write_u32::<LittleEndian>(s.len() as u32)?;
                writer.write_all(s.as_bytes())?;
            }
            PropertyValue::Boolean(b) => {
                writer.write_u32::<LittleEndian>(DataType::Boolean.to_u32())?;
                writer.write_u8(if *b { 1 } else { 0 })?;
            }
            PropertyValue::TimeStamp((secs, frac)) => {
                writer.write_u32::<LittleEndian>(DataType::TimeStamp.to_u32())?;
                writer.write_u64::<LittleEndian>(*frac)?;
                writer.write_i64::<LittleEndian>(*secs)?;
            }
        }
        Ok(())
    }
}

impl<'w> WriterGroup<'w> {
    /// Add a channel to this group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsWriter;
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// let mut g = w.add_group("DAQ")?;
    /// let mut ch = g.add_channel::<f64>("Voltage")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_channel<T: WritableType>(
        &mut self,
        name: impl Into<String>,
    ) -> Result<WriterChannel<'_, T>> {
        let name = name.into();

        if name.is_empty() {
            return Err(TdmsError::InvalidName("Channel name cannot be empty".into()));
        }

        let group = self
            .writer
            .groups
            .get_mut(&self.group_name)
            .ok_or_else(|| TdmsError::GroupNotFound(self.group_name.clone()))?;

        if !group.channels.contains_key(&name) {
            group.channels.insert(
                name.clone(),
                WriterChannelData {
                    name: name.clone(),
                    data_type: T::data_type(),
                    data: Vec::new(),
                    properties: IndexMap::new(),
                },
            );
        }

        Ok(WriterChannel {
            writer: self.writer,
            group_name: self.group_name.clone(),
            channel_name: name,
            _phantom: PhantomData,
        })
    }

    /// Add a property to this group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::{TdmsWriter, PropertyValue};
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// let mut g = w.add_group("DAQ")?;
    /// g.add_property("Description", PropertyValue::String("Voltage channel".into()))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        value: PropertyValue,
    ) -> Result<&mut Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName("Property name cannot be empty".into()));
        }
        let group = self
            .writer
            .groups
            .get_mut(&self.group_name)
            .ok_or_else(|| TdmsError::GroupNotFound(self.group_name.clone()))?;
        group.properties.insert(name, value);
        Ok(self)
    }
}

impl<'w, T: WritableType> WriterChannel<'w, T> {
    /// Write data to this channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::TdmsWriter;
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// let mut g = w.add_group("DAQ")?;
    /// let mut ch = g.add_channel::<f64>("Voltage")?;
    /// ch.write(&[1.0, 2.0, 3.0])?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write(&mut self, data: &[T]) -> Result<()> {
        let group = self
            .writer
            .groups
            .get_mut(&self.group_name)
            .ok_or_else(|| TdmsError::GroupNotFound(self.group_name.clone()))?;

        let channel = group
            .channels
            .get_mut(&self.channel_name)
            .ok_or_else(|| {
                TdmsError::ChannelNotFound(self.channel_name.clone(), self.group_name.clone())
            })?;

        T::write_to_buffer(data, &mut channel.data)?;
        Ok(())
    }

    /// Add a property to this channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms_rs::{TdmsWriter, PropertyValue};
    ///
    /// let mut w = TdmsWriter::create("out.tdms")?;
    /// let mut g = w.add_group("DAQ")?;
    /// let mut ch = g.add_channel::<f64>("Voltage")?;
    /// ch.add_property("Unit", PropertyValue::String("V".into()))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        value: PropertyValue,
    ) -> Result<&mut Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName("Property name cannot be empty".into()));
        }

        let group = self
            .writer
            .groups
            .get_mut(&self.group_name)
            .ok_or_else(|| TdmsError::GroupNotFound(self.group_name.clone()))?;

        let channel = group
            .channels
            .get_mut(&self.channel_name)
            .ok_or_else(|| {
                TdmsError::ChannelNotFound(self.channel_name.clone(), self.group_name.clone())
            })?;

        channel.properties.insert(name, value);
        Ok(self)
    }
}

/// Trait for types that can be written to TDMS files.
pub trait WritableType: Sized {
    fn data_type() -> DataType;
    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()>;
}

impl WritableType for f64 {
    fn data_type() -> DataType {
        DataType::DoubleFloat
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_f64::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for f32 {
    fn data_type() -> DataType {
        DataType::SingleFloat
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_f32::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for i8 {
    fn data_type() -> DataType {
        DataType::I8
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_i8(v)?;
        }
        Ok(())
    }
}

impl WritableType for i16 {
    fn data_type() -> DataType {
        DataType::I16
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_i16::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for i32 {
    fn data_type() -> DataType {
        DataType::I32
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_i32::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for i64 {
    fn data_type() -> DataType {
        DataType::I64
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_i64::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for u8 {
    fn data_type() -> DataType {
        DataType::U8
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        buffer.extend_from_slice(data);
        Ok(())
    }
}

impl WritableType for u16 {
    fn data_type() -> DataType {
        DataType::U16
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_u16::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for u32 {
    fn data_type() -> DataType {
        DataType::U32
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_u32::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for u64 {
    fn data_type() -> DataType {
        DataType::U64
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_u64::<LittleEndian>(v)?;
        }
        Ok(())
    }
}

impl WritableType for bool {
    fn data_type() -> DataType {
        DataType::Boolean
    }

    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_u8(if v { 1 } else { 0 })?;
        }
        Ok(())
    }
}
