use crate::error::{Result, TdmsError};
use crate::format::metadata::ParsingMetadata;
use crate::format::segment::Segment;
use crate::model::channel::{DataLocation, TdmsChannelData};
use crate::model::datatypes::{DataType, PropertyValue};
use crate::model::file::TdmsFileInner;
use crate::model::group::TdmsGroupData;
use byteorder::{LittleEndian, ReadBytesExt};
use indexmap::IndexMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

pub struct TdmsFile {
    pub(crate) inner: Arc<TdmsFileInner>,
}

pub struct TdmsGroup<'a> {
    pub(crate) file: &'a TdmsFile,
    pub(crate) data: &'a TdmsGroupData,
}

pub struct TdmsChannel<'a> {
    pub(crate) file: &'a TdmsFile,
    pub(crate) data: &'a TdmsChannelData,
}

pub struct TdmsSlice<'a> {
    pub(crate) data: ChannelData<'a>,
    pub(crate) dtype: DataType,
}

pub enum ChannelData<'a> {
    Mmap(&'a [u8]),
    Owned(Vec<u8>),
}

impl TdmsFile {
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

    pub fn group(&self, name: &str) -> Option<TdmsGroup<'_>> {
        let data = self.inner.groups.get(name)?;
        Some(TdmsGroup { file: self, data })
    }

    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.inner.properties.get(name)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.inner.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn groups(&self) -> impl Iterator<Item = TdmsGroup<'_>> {
        self.inner
            .groups
            .values()
            .map(move |data| TdmsGroup { file: self, data })
    }
}

impl<'a> TdmsGroup<'a> {
    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.data.properties.get(name)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.data.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn channel(&self, name: &str) -> Option<TdmsChannel<'a>> {
        let data = self.data.channels.get(name)?;
        Some(TdmsChannel {
            file: self.file,
            data,
        })
    }

    pub fn channels(&self) -> impl Iterator<Item = TdmsChannel<'a>> {
        let file = self.file;
        self.data
            .channels
            .values()
            .map(move |data| TdmsChannel { file, data })
    }
}

impl<'a> TdmsChannel<'a> {
    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn dtype(&self) -> DataType {
        self.data.dtype.clone()
    }

    pub fn len(&self) -> usize {
        self.data.len
    }

    pub fn is_empty(&self) -> bool {
        self.data.len == 0
    }

    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.data.properties.get(name)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.data.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn read(&self, range: Range<usize>) -> Result<TdmsSlice<'a>> {
        if range.end > self.data.len {
            return Err(TdmsError::InvalidRange(
                range.start,
                range.end,
                self.data.len,
            ));
        }

        let mut buffer = Vec::new();
        self.read_range_into(&range, &mut buffer)?;

        Ok(TdmsSlice {
            data: ChannelData::Owned(buffer),
            dtype: self.data.dtype.clone(),
        })
    }

    pub fn read_range(&self, range: Range<usize>) -> Result<TdmsSlice<'a>> {
        self.read(range)
    }

    pub fn read_all(&self) -> Result<TdmsSlice<'a>> {
        self.read(0..self.len())
    }

    pub fn read_into<T: Pod>(&self, range: Range<usize>, out: &mut [T]) -> Result<usize> {
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

    pub fn read_all_into<T: Pod>(&self, out: &mut [T]) -> Result<usize> {
        self.read_into(0..self.len(), out)
    }

    pub fn chunks(&'a self, chunk_size: usize) -> ChunkIterator<'a> {
        ChunkIterator {
            channel: self,
            chunk_size,
            offset: 0,
        }
    }

    pub fn iter_chunks(&'a self, chunk_size: usize) -> ChunkIterator<'a> {
        self.chunks(chunk_size)
    }

    pub fn timestamps(&self) -> Option<TimestampIterator> {
        let start_time = self.data.properties.get("wf_start_time").and_then(|v| {
            if let PropertyValue::Double(d) = v {
                Some(*d)
            } else {
                None
            }
        })?;

        let increment = self.data.properties.get("wf_increment").and_then(|v| {
            if let PropertyValue::Double(d) = v {
                Some(*d)
            } else {
                None
            }
        })?;

        Some(TimestampIterator {
            start: start_time,
            increment,
            index: 0,
            len: self.data.len,
        })
    }

    fn read_range_into(&self, range: &Range<usize>, buffer: &mut Vec<u8>) -> Result<()> {
        let file = File::open(&self.file.inner.path)?;
        let mut reader = BufReader::new(file);

        let itemsize = self.data.dtype.itemsize();
        let total_bytes = (range.end - range.start) * itemsize;
        buffer.reserve(total_bytes);

        let start_len = buffer.len();
        buffer.resize(start_len + total_bytes, 0);

        self.read_range_into_bytes(range, &mut buffer[start_len..], &mut reader)
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

pub struct ChunkIterator<'a> {
    channel: &'a TdmsChannel<'a>,
    chunk_size: usize,
    offset: usize,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = Result<TdmsSlice<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.channel.len() {
            return None;
        }

        let end = (self.offset + self.chunk_size).min(self.channel.len());
        let result = self.channel.read_range(self.offset..end);
        self.offset = end;

        Some(result)
    }
}

pub struct TimestampIterator {
    start: f64,
    increment: f64,
    index: usize,
    len: usize,
}

impl Iterator for TimestampIterator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let t = self.start + (self.index as f64) * self.increment;
        self.index += 1;
        Some(t)
    }
}

impl<'a> TdmsSlice<'a> {
    pub fn len(&self) -> usize {
        let bytes = match &self.data {
            ChannelData::Mmap(b) => b.len(),
            ChannelData::Owned(b) => b.len(),
        };
        bytes / self.dtype.itemsize()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_zero_copy(&self) -> bool {
        matches!(self.data, ChannelData::Mmap(_))
    }

    pub fn as_typed<T: Pod>(&self) -> Result<&[T]> {
        if std::mem::size_of::<T>() != self.dtype.itemsize() {
            return Err(TdmsError::TypeMismatch);
        }

        let bytes = match &self.data {
            ChannelData::Mmap(b) => *b,
            ChannelData::Owned(b) => b.as_slice(),
        };

        if bytes.is_empty() {
            return Ok(&[]);
        }

        if bytes.as_ptr() as usize % std::mem::align_of::<T>() != 0 {
            return Err(TdmsError::AlignmentError);
        }

        let ptr = bytes.as_ptr() as *const T;
        let len = bytes.len() / std::mem::size_of::<T>();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    }
}

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

        let mask_val = self.reader.read_u32::<LittleEndian>()?;
        let version = self.reader.read_u32::<LittleEndian>()?;
        let next_segment_offset = self.reader.read_u64::<LittleEndian>()?;
        let raw_data_offset = self.reader.read_u64::<LittleEndian>()?;

        let mask = crate::format::segment::Mask::new(mask_val);
        let mut objects = Vec::new();

        if mask.has_new_obj_list() {
            let count = self.reader.read_u32::<LittleEndian>()?;
            self.object_order.clear();

            for _ in 0..count {
                let path_len = self.reader.read_u32::<LittleEndian>()?;
                let mut path_bytes = vec![0u8; path_len as usize];
                self.reader.read_exact(&mut path_bytes)?;
                let path_str =
                    String::from_utf8(path_bytes).map_err(|_| TdmsError::StringEncoding)?;
                self.object_order.push(path_str.clone());

                let raw_data_index = self.reader.read_u32::<LittleEndian>()?;
                let mut raw_data_meta = None;
                let prop_count;

                if raw_data_index != 0 && raw_data_index != 0xFFFFFFFF {
                    let mut skipped = vec![0u8; raw_data_index as usize];
                    self.reader.read_exact(&mut skipped)?;

                    if raw_data_index >= 4 {
                        let mut slice = &skipped[0..4];
                        let type_code = slice.read_u32::<LittleEndian>()?;
                        let data_type = DataType::from_u32(type_code)?;

                        let mut count = 0;
                        let mut total_size = None;

                        if data_type == DataType::String {
                            if raw_data_index >= 16 {
                                let mut count_slice = &skipped[8..16];
                                count = count_slice.read_u64::<LittleEndian>()?;
                            }
                            if raw_data_index >= 24 {
                                let mut size_slice = &skipped[16..24];
                                total_size = Some(size_slice.read_u64::<LittleEndian>()?);
                            }
                            prop_count = self.reader.read_u32::<LittleEndian>()?;
                        } else {
                            if raw_data_index >= 16 {
                                let mut count_slice = &skipped[8..16];
                                count = count_slice.read_u64::<LittleEndian>()?;
                            }
                            let start = (raw_data_index - 4) as usize;
                            let mut end_slice = &skipped[start..];
                            prop_count = end_slice.read_u32::<LittleEndian>()?;
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
                    prop_count = self.reader.read_u32::<LittleEndian>()?;
                }

                let mut properties = std::collections::HashMap::new();
                for _ in 0..prop_count {
                    let key_len = self.reader.read_u32::<LittleEndian>()?;
                    let mut key_bytes = vec![0u8; key_len as usize];
                    self.reader.read_exact(&mut key_bytes)?;
                    let key =
                        String::from_utf8(key_bytes).map_err(|_| TdmsError::StringEncoding)?;
                    let type_code = self.reader.read_u32::<LittleEndian>()?;
                    let val =
                        crate::model::datatypes::DataType::from_u32(type_code).and_then(|dt| {
                            match dt {
                                DataType::I8 => Ok(PropertyValue::I8(self.reader.read_i8()?)),
                                DataType::I16 => {
                                    Ok(PropertyValue::I16(self.reader.read_i16::<LittleEndian>()?))
                                }
                                DataType::I32 => {
                                    Ok(PropertyValue::I32(self.reader.read_i32::<LittleEndian>()?))
                                }
                                DataType::I64 => {
                                    Ok(PropertyValue::I64(self.reader.read_i64::<LittleEndian>()?))
                                }
                                DataType::U8 => Ok(PropertyValue::U8(self.reader.read_u8()?)),
                                DataType::U16 => {
                                    Ok(PropertyValue::U16(self.reader.read_u16::<LittleEndian>()?))
                                }
                                DataType::U32 => {
                                    Ok(PropertyValue::U32(self.reader.read_u32::<LittleEndian>()?))
                                }
                                DataType::U64 => {
                                    Ok(PropertyValue::U64(self.reader.read_u64::<LittleEndian>()?))
                                }
                                DataType::Float => Ok(PropertyValue::Float(
                                    self.reader.read_f32::<LittleEndian>()?,
                                )),
                                DataType::Double => Ok(PropertyValue::Double(
                                    self.reader.read_f64::<LittleEndian>()?,
                                )),
                                DataType::Boolean => {
                                    Ok(PropertyValue::Boolean(self.reader.read_u8()? != 0))
                                }
                                DataType::String => {
                                    let len = self.reader.read_u32::<LittleEndian>()?;
                                    let mut buf = vec![0u8; len as usize];
                                    self.reader.read_exact(&mut buf)?;
                                    let s = String::from_utf8(buf)
                                        .map_err(|_| TdmsError::StringEncoding)?;
                                    Ok(PropertyValue::String(s))
                                }
                                DataType::TimeStamp => {
                                    let fraction = self.reader.read_u64::<LittleEndian>()?;
                                    let seconds = self.reader.read_i64::<LittleEndian>()?;
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
                    let size = if let Some(s) = meta.data_type.size_of_old() {
                        s * meta.number_of_values
                    } else {
                        meta.total_size_bytes.unwrap_or(0)
                    };
                    obj.data_location = Some(crate::format::metadata::DataLocation {
                        offset: current_raw_offset,
                        number_of_values: meta.number_of_values,
                        data_type: meta.data_type.clone(),
                        total_size_bytes: meta.total_size_bytes,
                    });
                    current_raw_offset += size;
                }
            } else if obj.raw_data_index == 0 {
                if let Some(meta) = self.active_meta.get(&path_str) {
                    if meta.number_of_values > 0 {
                        let size = if let Some(s) = meta.data_type.size_of_old() {
                            s * meta.number_of_values
                        } else {
                            meta.total_size_bytes.unwrap_or(0)
                        };
                        obj.data_location = Some(crate::format::metadata::DataLocation {
                            offset: current_raw_offset,
                            number_of_values: meta.number_of_values,
                            data_type: meta.data_type.clone(),
                            total_size_bytes: meta.total_size_bytes,
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
            version,
            next_segment_offset,
            raw_data_offset,
            toc_mask: mask.convert(),
            objects,
        })
    }
}

impl DataType {
    fn size_of_old(&self) -> Option<u64> {
        match self {
            DataType::I8 | DataType::U8 | DataType::Boolean => Some(1),
            DataType::I16 | DataType::U16 => Some(2),
            DataType::I32 | DataType::U32 | DataType::Float => Some(4),
            DataType::I64 | DataType::U64 | DataType::Double => Some(8),
            DataType::TimeStamp => Some(16),
            DataType::String => None,
        }
    }
}
