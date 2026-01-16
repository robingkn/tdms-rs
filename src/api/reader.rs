use crate::error::{Result, TdmsError};
use crate::format::metadata::ParsingMetadata;
use crate::format::segment::Segment;
use crate::io::ext::TdmsReadExt;
use crate::model::channel::{DataLocation, TdmsChannelData};
use crate::model::datatypes::{DataType, PropertyValue};
use crate::model::file::TdmsFileInner;
use crate::model::group::TdmsGroupData;
use indexmap::IndexMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

/// A TDMS file handle for reading.
///
/// This struct indexes the file structure (groups, channels, properties) on open
/// without loading raw data into memory.
pub struct TdmsFile {
    pub(crate) inner: Arc<TdmsFileInner>,
}

/// A group within a TDMS file.
pub struct TdmsGroup<'a> {
    pub(crate) file: &'a TdmsFile,
    pub(crate) data: &'a TdmsGroupData,
}

/// A channel within a TDMS group.
pub struct TdmsChannel<'a> {
    pub(crate) file: &'a TdmsFile,
    pub(crate) data: &'a TdmsChannelData,
}

impl TdmsFile {
    /// Open a TDMS file for reading.
    ///
    /// This parses and indexes all segment metadata eagerly, but does not read
    /// raw channel data until it is requested.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mut reader = TdmsReaderInternal::new(BufReader::new(file));

        let mut groups = IndexMap::new();
        let mut file_properties = IndexMap::new();

        loop {
            let segment = match reader.read_segment() {
                Ok(s) => s,
                Err(TdmsError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(TdmsError::Io(e))
                    if e.kind() == std::io::ErrorKind::Other
                        && e.to_string().contains("UnexpectedEof") =>
                {
                    break
                }
                Err(e) => return Err(e),
            };

            for obj in segment.objects {
                if let Some(g_name) = obj.path.group_name() {
                    let group = groups
                        .entry(g_name.to_string())
                        .or_insert_with(|| TdmsGroupData {
                            name: g_name.to_string(),
                            channels: IndexMap::new(),
                            properties: IndexMap::new(),
                        });

                    if let Some(c_name) = obj.path.channel_name() {
                        let channel =
                            group.channels.entry(c_name.to_string()).or_insert_with(|| {
                                TdmsChannelData {
                                    name: c_name.to_string(),
                                    dtype: DataType::Double,
                                    len: 0,
                                    data_locations: Vec::new(),
                                    properties: IndexMap::new(),
                                }
                            });

                        channel.properties.extend(obj.properties);

                        if let Some(loc) = obj.data_location {
                            channel.len += loc.number_of_values as usize;
                            channel.data_locations.push(DataLocation {
                                offset: loc.offset,
                                number_of_values: loc.number_of_values,
                            });
                        }

                        if let Some(meta) = obj.raw_data_meta {
                            channel.dtype = meta.data_type;
                        }
                    } else {
                        group.properties.extend(obj.properties);
                    }
                } else if obj.path.is_root() {
                    file_properties.extend(obj.properties);
                }
            }
        }

        Ok(Self {
            inner: Arc::new(TdmsFileInner {
                path: path.to_path_buf(),
                groups,
                properties: file_properties,
            }),
        })
    }

    /// Look up a group by name.
    pub fn group(&self, name: &str) -> Option<TdmsGroup<'_>> {
        let data = self.inner.groups.get(name)?;
        Some(TdmsGroup { file: self, data })
    }

    /// Look up a file-level property by name.
    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.inner.properties.get(name)
    }

    /// Iterate over all file-level properties.
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.inner.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over all groups in the file.
    pub fn groups(&self) -> impl Iterator<Item = TdmsGroup<'_>> {
        self.inner
            .groups
            .values()
            .map(move |data| TdmsGroup { file: self, data })
    }
}

impl<'a> TdmsGroup<'a> {
    /// Return the group name.
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Look up a group-level property by name.
    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.data.properties.get(name)
    }

    /// Iterate over all group-level properties.
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.data.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Look up a channel by name.
    pub fn channel(&self, name: &str) -> Option<TdmsChannel<'a>> {
        let data = self.data.channels.get(name)?;
        Some(TdmsChannel {
            file: self.file,
            data,
        })
    }

    /// Iterate over all channels in the group.
    pub fn channels(&self) -> impl Iterator<Item = TdmsChannel<'a>> {
        let file = self.file;
        self.data
            .channels
            .values()
            .map(move |data| TdmsChannel { file, data })
    }
}

impl<'a> TdmsChannel<'a> {
    /// Return the channel name.
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Return the channel data type.
    pub fn dtype(&self) -> DataType {
        self.data.dtype.clone()
    }

    /// Return the number of values in the channel.
    pub fn len(&self) -> usize {
        self.data.len
    }

    /// Returns `true` if the channel contains no values.
    pub fn is_empty(&self) -> bool {
        self.data.len == 0
    }

    /// Look up a channel-level property by name.
    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.data.properties.get(name)
    }

    /// Iterate over all channel-level properties.
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.data.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Read a range of values into the provided output buffer.
    ///
    /// The element type `T` must match the TDMS channel element size.
    pub fn read<T: Pod>(&self, range: Range<usize>, out: &mut [T]) -> Result<usize> {
        if range.end > self.data.len {
            return Err(TdmsError::InvalidRange(
                range.start,
                range.end,
                self.data.len,
            ));
        }
        if std::mem::size_of::<T>() != self.data.dtype.itemsize() {
            return Err(TdmsError::TypeMismatch);
        }
        let requested = range.end - range.start;
        if out.len() < requested {
            return Err(TdmsError::InvalidFormat(
                "output buffer too small for requested range".to_string(),
            ));
        }

        let out_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                out.as_mut_ptr() as *mut u8,
                requested * std::mem::size_of::<T>(),
            )
        };

        let file = File::open(&self.file.inner.path)?;
        let mut reader = BufReader::new(file);
        self.read_range_into_bytes(&range, out_bytes, &mut reader)?;
        Ok(requested)
    }


    fn read_range_into_bytes<R: std::io::Read + std::io::Seek>(
        &self,
        range: &Range<usize>,
        out: &mut [u8],
        reader: &mut R,
    ) -> Result<()> {
        let itemsize = self.data.dtype.itemsize();
        let total_bytes = (range.end - range.start) * itemsize;
        if out.len() != total_bytes {
            return Err(TdmsError::InvalidFormat(
                "output buffer length must exactly match requested byte length".to_string(),
            ));
        }

        let mut remaining = range.end - range.start;
        let mut current_offset = range.start;
        let mut out_cursor = 0;

        for loc in &self.data.data_locations {
            let loc_end = loc.number_of_values as usize;

            if current_offset >= loc_end {
                current_offset -= loc_end;
                continue;
            }

            let read_start = current_offset;
            let read_end = loc_end.min(current_offset + remaining);
            let read_count = read_end - read_start;

            if read_count == 0 {
                break;
            }

            let byte_offset = loc.offset + (read_start * itemsize) as u64;
            reader.seek(SeekFrom::Start(byte_offset))?;

            let read_bytes = read_count * itemsize;
            reader.read_exact(&mut out[out_cursor..out_cursor + read_bytes])?;
            out_cursor += read_bytes;

            remaining -= read_count;
            current_offset = 0;

            if remaining == 0 {
                break;
            }
        }

        Ok(())
    }
}


/// Marker trait for plain-old-data types supported by [`TdmsChannel::read`].
pub trait Pod: Copy {}
impl Pod for i8 {}
impl Pod for u8 {}
impl Pod for i16 {}
impl Pod for u16 {}
impl Pod for i32 {}
impl Pod for u32 {}
impl Pod for i64 {}
impl Pod for u64 {}
impl Pod for f32 {}
impl Pod for f64 {}
impl Pod for bool {}

struct TdmsReaderInternal<R: Read + Seek> {
    reader: R,
    active_meta: std::collections::HashMap<String, crate::format::metadata::RawDataMeta>,
    object_order: Vec<String>,
}

impl<R: Read + Seek> TdmsReaderInternal<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            active_meta: std::collections::HashMap::new(),
            object_order: Vec::new(),
        }
    }

    fn read_segment(&mut self) -> Result<Segment> {
        let start_pos = self.reader.stream_position()?;
        let mut lead_in = [0u8; 4];
        self.reader.read_exact(&mut lead_in)?;

        if &lead_in != b"TDSm" {
            return Err(TdmsError::InvalidSignature);
        }

        let mask_val = self.reader.read_u32()?;
        let version = self.reader.read_u32()?;
        let next_segment_offset = self.reader.read_u64()?;
        let raw_data_offset = self.reader.read_u64()?;

        let mask = crate::format::segment::Mask::new(mask_val);
        let mut objects = Vec::new();

        if mask.has_new_obj_list() {
            let count = self.reader.read_u32()?;
            self.object_order.clear();

            for _ in 0..count {
                let path_len = self.reader.read_u32()?;
                let mut path_bytes = vec![0u8; path_len as usize];
                self.reader.read_exact(&mut path_bytes)?;
                let path_str =
                    String::from_utf8(path_bytes).map_err(|_| TdmsError::StringEncoding)?;
                self.object_order.push(path_str.clone());

                let raw_data_index = self.reader.read_u32()?;
                let mut raw_data_meta = None;
                let prop_count;

                if raw_data_index != 0 && raw_data_index != 0xFFFFFFFF {
                    let mut skipped = vec![0u8; raw_data_index as usize];
                    self.reader.read_exact(&mut skipped)?;

                    if raw_data_index >= 4 {
                        let mut slice = &skipped[0..4];
                        let type_code = slice.read_u32()?;
                        let data_type = DataType::from_u32(type_code)?;

                        let mut count = 0;
                        let mut total_size = None;

                        if data_type == DataType::String {
                            if raw_data_index >= 16 {
                                let mut count_slice = &skipped[8..16];
                                count = count_slice.read_u64()?;
                            }
                            if raw_data_index >= 24 {
                                let mut size_slice = &skipped[16..24];
                                total_size = Some(size_slice.read_u64()?);
                            }
                            prop_count = self.reader.read_u32()?;
                        } else {
                            if raw_data_index >= 16 {
                                let mut count_slice = &skipped[8..16];
                                count = count_slice.read_u64()?;
                            }
                            let start = (raw_data_index - 4) as usize;
                            let mut end_slice = &skipped[start..];
                            prop_count = end_slice.read_u32()?;
                        }

                        raw_data_meta = Some(crate::format::metadata::RawDataMeta {
                            data_type,
                            number_of_values: count,
                            total_size_bytes: total_size,
                        });
                    } else {
                        prop_count = 0;
                    }
                } else {
                    prop_count = self.reader.read_u32()?;
                }

                let mut properties = std::collections::HashMap::new();
                for _ in 0..prop_count {
                    let key_len = self.reader.read_u32()?;
                    let mut key_bytes = vec![0u8; key_len as usize];
                    self.reader.read_exact(&mut key_bytes)?;
                    let key =
                        String::from_utf8(key_bytes).map_err(|_| TdmsError::StringEncoding)?;
                    let type_code = self.reader.read_u32()?;
                    let val =
                        crate::model::datatypes::DataType::from_u32(type_code).and_then(|dt| {
                            match dt {
                                DataType::I8 => Ok(PropertyValue::I8(self.reader.read_i8()?)),
                                DataType::I16 => Ok(PropertyValue::I16(self.reader.read_i16()?)),
                                DataType::I32 => Ok(PropertyValue::I32(self.reader.read_i32()?)),
                                DataType::I64 => Ok(PropertyValue::I64(self.reader.read_i64()?)),
                                DataType::U8 => Ok(PropertyValue::U8(self.reader.read_u8()?)),
                                DataType::U16 => Ok(PropertyValue::U16(self.reader.read_u16()?)),
                                DataType::U32 => Ok(PropertyValue::U32(self.reader.read_u32()?)),
                                DataType::U64 => Ok(PropertyValue::U64(self.reader.read_u64()?)),
                                DataType::Float => {
                                    Ok(PropertyValue::Float(self.reader.read_f32()?))
                                }
                                DataType::Double => {
                                    Ok(PropertyValue::Double(self.reader.read_f64()?))
                                }
                                DataType::Boolean => {
                                    Ok(PropertyValue::Boolean(self.reader.read_u8()? != 0))
                                }
                                DataType::String => {
                                    let len = self.reader.read_u32()?;
                                    let mut buf = vec![0u8; len as usize];
                                    self.reader.read_exact(&mut buf)?;
                                    let s = String::from_utf8(buf)
                                        .map_err(|_| TdmsError::StringEncoding)?;
                                    Ok(PropertyValue::String(s))
                                }
                                DataType::TimeStamp => {
                                    let fraction = self.reader.read_u64()?;
                                    let seconds = self.reader.read_i64()?;
                                    Ok(PropertyValue::TimeStamp((seconds, fraction)))
                                }
                            }
                        })?;
                    properties.insert(key, val);
                }

                objects.push(ParsingMetadata {
                    path: crate::format::metadata::ObjectPath::new(path_str),
                    raw_data_index,
                    properties,
                    raw_data_meta,
                    data_location: None,
                });
            }
        } else {
            for path_str in &self.object_order {
                objects.push(ParsingMetadata {
                    path: crate::format::metadata::ObjectPath::new(path_str.clone()),
                    raw_data_index: 0,
                    properties: std::collections::HashMap::new(),
                    raw_data_meta: None,
                    data_location: None,
                });
            }
        }

        let mut current_raw_offset = start_pos + 28 + raw_data_offset;
        for obj in &mut objects {
            let path_str = obj.path.raw.clone();
            if let Some(meta) = &obj.raw_data_meta {
                self.active_meta.insert(path_str.clone(), meta.clone());
                if meta.number_of_values > 0 {
                    let size = (meta.data_type.itemsize() as u64) * meta.number_of_values;
                    obj.data_location = Some(crate::format::metadata::DataLocation {
                        offset: current_raw_offset,
                        number_of_values: meta.number_of_values,
                        _data_type: meta.data_type.clone(),
                        _total_size_bytes: meta.total_size_bytes,
                    });
                    current_raw_offset += size;
                }
            } else if obj.raw_data_index == 0 {
                if let Some(meta) = self.active_meta.get(&path_str) {
                    if meta.number_of_values > 0 {
                        let size = (meta.data_type.itemsize() as u64) * meta.number_of_values;
                        obj.data_location = Some(crate::format::metadata::DataLocation {
                            offset: current_raw_offset,
                            number_of_values: meta.number_of_values,
                            _data_type: meta.data_type.clone(),
                            _total_size_bytes: meta.total_size_bytes,
                        });
                        current_raw_offset += size;
                    }
                }
            }
        }

        let target_pos = if next_segment_offset != 0xFFFFFFFFFFFFFFFF {
            start_pos + 28 + next_segment_offset
        } else {
            current_raw_offset
        };

        let current_pos = self.reader.stream_position()?;
        if current_pos != target_pos {
            self.reader.seek(SeekFrom::Start(target_pos))?;
        }

        Ok(Segment {
            _version: version,
            _next_segment_offset: next_segment_offset,
            _raw_data_offset: raw_data_offset,
            _toc_mask: mask.convert(),
            objects,
        })
    }
}
