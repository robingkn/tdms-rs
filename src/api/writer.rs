use crate::error::{Result, TdmsError};
use crate::io::ext::TdmsWriteExt;
use crate::model::datatypes::{DataType, PropertyValue};
use indexmap::IndexMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A TDMS file writer.
///
/// Data is batched in memory and written to disk when `close()` is called.
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

pub struct WriterGroup<'w> {
    pub(crate) writer: &'w mut TdmsWriter,
    pub(crate) group_name: String,
}

/// A typed writer handle for a single channel.
pub struct WriterChannel<'w, T> {
    pub(crate) writer: &'w mut TdmsWriter,
    pub(crate) group_name: String,
    pub(crate) channel_name: String,
    pub(crate) _phantom: PhantomData<T>,
}

impl TdmsWriter {
    /// Create a new TDMS writer targeting the given output path.
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            groups: IndexMap::new(),
            properties: IndexMap::new(),
            closed: false,
        })
    }

    /// Add (or look up) a group in the output file.
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

    /// Add a file-level property.
    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        value: PropertyValue,
    ) -> Result<&mut Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName(
                "Property name cannot be empty".into(),
            ));
        }
        self.properties.insert(name, value);
        Ok(self)
    }

    /// Write the TDMS file to disk.
    pub fn close(mut self) -> Result<()> {
        if self.closed {
            return Err(TdmsError::WriterClosed);
        }
        self.write_file()?;
        self.closed = true;
        Ok(())
    }

    /// Abort writing and delete the output file if it already exists.
    pub fn abort(self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    fn write_file(&mut self) -> Result<()> {
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(b"TDSm")?;
        let toc = 0x0E;
        writer.write_u32(toc)?;
        writer.write_u32(4712)?;

        let segment_offset_pos = writer.stream_position()?;
        writer.write_u64(0xFFFFFFFFFFFFFFFF)?;
        writer.write_u64(0)?;

        let mut object_count = 0;
        if !self.properties.is_empty() {
            object_count += 1;
        }
        for group in self.groups.values() {
            object_count += 1;
            object_count += group.channels.len() as u32;
        }

        writer.write_u32(object_count)?;

        if !self.properties.is_empty() {
            self.write_object_internal(&mut writer, "/", &self.properties, None)?;
        }

        for group in self.groups.values() {
            let group_path = format!("/'{}'", group.name);
            self.write_object_internal(&mut writer, &group_path, &group.properties, None)?;

            for channel in group.channels.values() {
                let channel_path = format!("/'{}'/'{}'", group.name, channel.name);
                self.write_object_internal(
                    &mut writer,
                    &channel_path,
                    &channel.properties,
                    Some((&channel.data_type, channel.data.len())),
                )?;
            }
        }

        let raw_data_offset = writer.stream_position()?;
        for group in self.groups.values() {
            for channel in group.channels.values() {
                writer.write_all(&channel.data)?;
            }
        }

        let end_pos = writer.stream_position()?;
        writer.seek(std::io::SeekFrom::Start(segment_offset_pos + 8))?;
        writer.write_u64(raw_data_offset - 28)?;
        writer.seek(std::io::SeekFrom::Start(end_pos))?;

        writer.flush()?;
        Ok(())
    }

    fn write_object_internal(
        &self,
        writer: &mut BufWriter<File>,
        path: &str,
        properties: &IndexMap<String, PropertyValue>,
        raw_data: Option<(&DataType, usize)>,
    ) -> Result<()> {
        writer.write_u32(path.len() as u32)?;
        writer.write_all(path.as_bytes())?;

        let raw_data_index = if raw_data.is_some() {
            20_u32
        } else {
            0xFFFFFFFF_u32
        };
        writer.write_u32(raw_data_index)?;

        if let Some((dtype, byte_len)) = raw_data {
            writer.write_u32(dtype.to_u32())?;
            writer.write_u32(1)?;
            let count = byte_len / dtype.itemsize();
            writer.write_u64(count as u64)?;
        }

        writer.write_u32(properties.len() as u32)?;
        for (key, value) in properties {
            writer.write_u32(key.len() as u32)?;
            writer.write_all(key.as_bytes())?;
            self.write_property_value_internal(writer, value)?;
        }
        Ok(())
    }

    fn write_property_value_internal(
        &self,
        writer: &mut BufWriter<File>,
        value: &PropertyValue,
    ) -> Result<()> {
        match value {
            PropertyValue::I8(v) => {
                writer.write_u32(DataType::I8.to_u32())?;
                writer.write_i8(*v)?;
            }
            PropertyValue::I16(v) => {
                writer.write_u32(DataType::I16.to_u32())?;
                writer.write_i16(*v)?;
            }
            PropertyValue::I32(v) => {
                writer.write_u32(DataType::I32.to_u32())?;
                writer.write_i32(*v)?;
            }
            PropertyValue::I64(v) => {
                writer.write_u32(DataType::I64.to_u32())?;
                writer.write_i64(*v)?;
            }
            PropertyValue::U8(v) => {
                writer.write_u32(DataType::U8.to_u32())?;
                writer.write_u8(*v)?;
            }
            PropertyValue::U16(v) => {
                writer.write_u32(DataType::U16.to_u32())?;
                writer.write_u16(*v)?;
            }
            PropertyValue::U32(v) => {
                writer.write_u32(DataType::U32.to_u32())?;
                writer.write_u32(*v)?;
            }
            PropertyValue::U64(v) => {
                writer.write_u32(DataType::U64.to_u32())?;
                writer.write_u64(*v)?;
            }
            PropertyValue::Float(v) => {
                writer.write_u32(DataType::Float.to_u32())?;
                writer.write_f32(*v)?;
            }
            PropertyValue::Double(v) => {
                writer.write_u32(DataType::Double.to_u32())?;
                writer.write_f64(*v)?;
            }
            PropertyValue::String(s) => {
                writer.write_u32(DataType::String.to_u32())?;
                writer.write_u32(s.len() as u32)?;
                writer.write_all(s.as_bytes())?;
            }
            PropertyValue::Boolean(b) => {
                writer.write_u32(DataType::Boolean.to_u32())?;
                writer.write_u8(if *b { 1 } else { 0 })?;
            }
            PropertyValue::TimeStamp((secs, frac)) => {
                writer.write_u32(DataType::TimeStamp.to_u32())?;
                writer.write_u64(*frac)?;
                writer.write_i64(*secs)?;
            }
        }
        Ok(())
    }
}

impl<'w> WriterGroup<'w> {
    /// Add (or look up) a channel within this group.
    pub fn add_channel<T: WritableType>(
        &mut self,
        name: impl Into<String>,
    ) -> Result<WriterChannel<'_, T>> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName(
                "Channel name cannot be empty".into(),
            ));
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

    /// Add a group-level property.
    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        value: PropertyValue,
    ) -> Result<&mut Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName(
                "Property name cannot be empty".into(),
            ));
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
    /// Append the provided data to the channel.
    pub fn write(&mut self, data: &[T]) -> Result<()> {
        let group = self
            .writer
            .groups
            .get_mut(&self.group_name)
            .ok_or_else(|| TdmsError::GroupNotFound(self.group_name.clone()))?;
        let channel = group.channels.get_mut(&self.channel_name).ok_or_else(|| {
            TdmsError::ChannelNotFound(self.channel_name.clone(), self.group_name.clone())
        })?;
        T::write_to_buffer(data, &mut channel.data)?;
        Ok(())
    }

    /// Add a channel-level property.
    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        value: PropertyValue,
    ) -> Result<&mut Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(TdmsError::InvalidName(
                "Property name cannot be empty".into(),
            ));
        }
        let group = self
            .writer
            .groups
            .get_mut(&self.group_name)
            .ok_or_else(|| TdmsError::GroupNotFound(self.group_name.clone()))?;
        let channel = group.channels.get_mut(&self.channel_name).ok_or_else(|| {
            TdmsError::ChannelNotFound(self.channel_name.clone(), self.group_name.clone())
        })?;
        channel.properties.insert(name, value);
        Ok(self)
    }
}

/// Trait implemented for element types that can be written as TDMS channel data.
pub trait WritableType: Sized {
    fn data_type() -> DataType;
    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()>;
}

impl WritableType for f64 {
    fn data_type() -> DataType {
        DataType::Double
    }
    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_f64(v)?;
        }
        Ok(())
    }
}

impl WritableType for f32 {
    fn data_type() -> DataType {
        DataType::Float
    }
    fn write_to_buffer(data: &[Self], buffer: &mut Vec<u8>) -> Result<()> {
        for &v in data {
            buffer.write_f32(v)?;
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
            buffer.write_i16(v)?;
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
            buffer.write_i32(v)?;
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
            buffer.write_i64(v)?;
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
            buffer.write_u16(v)?;
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
            buffer.write_u32(v)?;
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
            buffer.write_u64(v)?;
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
