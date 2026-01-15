//! # tdms
//!
//! A pure Rust library for reading and writing National Instruments TDMS files.
//!
//! ## Reading Example
//!
//! ```no_run
//! use tdms::TdmsFile;
//!
//! let f = TdmsFile::open("data.tdms")?;
//! let ch = f.group("G").unwrap().channel("C").unwrap();
//! let slice = ch.read(0..100)?;
//! let data: &[f64] = slice.as_typed()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Writing Example
//!
//! ```no_run
//! use tdms::TdmsWriter;
//!
//! let mut w = TdmsWriter::create("out.tdms")?;
//! let mut g = w.add_group("DAQ")?;
//! let mut ch = g.add_channel::<f64>("Voltage")?;
//! ch.write(&[1.0, 2.0, 3.0])?;
//! w.close()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod datatypes;
mod error;
mod metadata;
mod reader;
mod segment;
mod writer;

pub use datatypes::PropertyValue;
pub use error::{Result, TdmsError};
pub use writer::{TdmsWriter, WriterGroup, WriterChannel};

use indexmap::IndexMap;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A TDMS file handle for reading.
///
/// `TdmsFile` is `Send + Sync` and can be shared across threads using `Arc`.
pub struct TdmsFile {
    inner: Arc<TdmsFileInner>,
}

struct TdmsFileInner {
    path: PathBuf,
    groups: IndexMap<String, TdmsGroupData>,
    properties: IndexMap<String, PropertyValue>,
}

struct TdmsGroupData {
    name: String,
    channels: IndexMap<String, TdmsChannelData>,
    properties: IndexMap<String, PropertyValue>,
}

struct TdmsChannelData {
    name: String,
    dtype: TdmsDType,
    len: usize,
    data_locations: Vec<metadata::DataLocation>,
    properties: IndexMap<String, PropertyValue>,
}

/// A view into a group within a TDMS file.
pub struct TdmsGroup<'a> {
    file: &'a TdmsFile,
    data: &'a TdmsGroupData,
}

/// A view into a channel within a TDMS group.
pub struct TdmsChannel<'a> {
    file: &'a TdmsFile,
    data: &'a TdmsChannelData,
}

/// Data type information for a TDMS channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TdmsDType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Bool,
    String,
    TimeStamp,
}

/// A slice of channel data, potentially zero-copy.
pub struct TdmsSlice<'a> {
    data: ChannelData<'a>,
    dtype: TdmsDType,
}

/// Internal representation of channel data.
pub enum ChannelData<'a> {
    /// Memory-mapped data (zero-copy)
    Mmap(&'a [u8]),
    /// Owned data (fallback)
    Owned(Vec<u8>),
}

impl TdmsFile {
    /// Open a TDMS file for reading.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let file = TdmsFile::open("data.tdms")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mut reader = reader::TdmsReader::new(BufReader::new(file));

        let mut groups = IndexMap::new();
        let mut file_properties = IndexMap::new();

        loop {
            let segment = match reader.read_segment() {
                Ok(s) => s,
                Err(TdmsError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
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
                        let channel = group
                            .channels
                            .entry(c_name.to_string())
                            .or_insert_with(|| TdmsChannelData {
                                name: c_name.to_string(),
                                dtype: TdmsDType::F64, // Will be updated
                                len: 0,
                                data_locations: Vec::new(),
                                properties: IndexMap::new(),
                            });

                        channel.properties.extend(obj.properties);

                        if let Some(loc) = obj.data_location {
                            channel.len += loc.number_of_values as usize;
                            channel.data_locations.push(loc);
                        }

                        if let Some(meta) = obj.raw_data_meta {
                            channel.dtype = dtype_from_internal(&meta.data_type);
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

    /// Get a group by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let g = f.group("Sensors").unwrap();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn group(&self, name: &str) -> Option<TdmsGroup> {
        let data = self.inner.groups.get(name)?;
        Some(TdmsGroup { file: self, data })
    }

    /// Get a property by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let prop = f.property("prop_name")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.inner.properties.get(name)
    }

    /// Iterate over all properties in the file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// for (name, prop) in f.properties() {
    ///     println!("{}: {:?}", name, prop);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.inner
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate over all groups in the file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// for g in f.groups() {
    ///     println!("Group: {}", g.name());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn groups(&self) -> impl Iterator<Item = TdmsGroup> {
        self.inner
            .groups
            .values()
            .map(move |data| TdmsGroup { file: self, data })
    }
}

impl<'a> TdmsGroup<'a> {
    /// Get the name of this group.
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Get a property by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let g = f.group("Sensors").unwrap();
    /// let prop = g.property("prop_name");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.data.properties.get(name)
    }

    /// Iterate over all properties in this group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let g = f.group("Sensors").unwrap();
    /// for (name, prop) in g.properties() {
    ///     println!("{}: {:?}", name, prop);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.data
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v))
    }

    /// Get a channel by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let g = f.group("Sensors").unwrap();
    /// let ch = g.channel("Temperature").unwrap();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn channel(&self, name: &str) -> Option<TdmsChannel<'a>> {
        let data = self.data.channels.get(name)?;
        Some(TdmsChannel {
            file: self.file,
            data,
        })
    }

    /// Iterate over all channels in this group.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let g = f.group("Sensors").unwrap();
    /// for ch in g.channels() {
    ///     println!("  Channel: {}", ch.name());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn channels(&self) -> impl Iterator<Item = TdmsChannel<'a>> {
        let file = self.file;
        self.data
            .channels
            .values()
            .map(move |data| TdmsChannel { file, data })
    }
}

impl<'a> TdmsChannel<'a> {
    /// Get the name of this channel.
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Get the data type of this channel.
    pub fn dtype(&self) -> TdmsDType {
        self.data.dtype
    }

    /// Get the number of samples in this channel.
    pub fn len(&self) -> usize {
        self.data.len
    }

    /// Get a property by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("Sensors").unwrap().channel("Temperature").unwrap();
    /// let prop = ch.property("prop_name");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn property(&self, name: &str) -> Option<&PropertyValue> {
        self.data.properties.get(name)
    }

    /// Iterate over all properties in this channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("Sensors").unwrap().channel("Temperature").unwrap();
    /// for (name, prop) in ch.properties() {
    ///     println!("{}: {:?}", name, prop);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)> {
        self.data
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v))
    }

    /// Check if the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.data.len == 0
    }

    /// Read a range of data from this channel.
    ///
    /// This is the primary method for reading channel data. Uses half-open range `[start..end)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("G").unwrap().channel("C").unwrap();
    /// let slice = ch.read(0..100)?;
    /// let data: &[f64] = slice.as_typed()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read(&self, range: Range<usize>) -> Result<TdmsSlice<'a>> {
        self.read_range(range)
    }

    /// Read a range of data from this channel.
    ///
    /// Alias for [`read`](Self::read).
    pub fn read_range(&self, range: Range<usize>) -> Result<TdmsSlice<'a>> {
        if range.end > self.data.len {
            return Err(TdmsError::InvalidRange(range.start, range.end, self.data.len));
        }

        // For now, always use owned data
        // TODO: Implement zero-copy mmap path
        let mut buffer = Vec::new();
        self.read_range_into(&range, &mut buffer)?;

        Ok(TdmsSlice {
            data: ChannelData::Owned(buffer),
            dtype: self.data.dtype,
        })
    }

    /// Read a range of data from this channel into a buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("G").unwrap().channel("C").unwrap();
    /// let mut buffer = [0.0; 100];
    /// let count = ch.read_into(0..100, &mut buffer)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read_into<T: Pod>(&self, range: Range<usize>, out: &mut [T]) -> Result<usize> {
        if range.end > self.data.len {
            return Err(TdmsError::InvalidRange(range.start, range.end, self.data.len));
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
            std::slice::from_raw_parts_mut(out.as_mut_ptr() as *mut u8, requested * std::mem::size_of::<T>())
        };

        let file = File::open(&self.file.inner.path)?;
        let mut reader = BufReader::new(file);
        self.read_range_into_bytes(&range, out_bytes, &mut reader)?;
        Ok(requested)
    }

    /// Read all data from this channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("G").unwrap().channel("C").unwrap();
    /// let slice = ch.read_all()?;
    /// let data: &[f64] = slice.as_typed()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read_all(&self) -> Result<TdmsSlice<'a>> {
        self.read_range(0..self.len())
    }

    /// Read all data from this channel into a buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("G").unwrap().channel("C").unwrap();
    /// let mut buffer = [0.0; 100];
    /// let count = ch.read_all_into(&mut buffer)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read_all_into<T: Pod>(&self, out: &mut [T]) -> Result<usize> {
        self.read_into(0..self.len(), out)
    }

    /// Iterate over chunks of data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("G").unwrap().channel("C").unwrap();
    /// for chunk in ch.chunks(10_000) {
    ///     let slice = chunk?;
    ///     println!("chunk len = {}", slice.len());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Get timestamps for this channel if available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("DAQ").unwrap().channel("Voltage").unwrap();
    /// if let Some(ts) = ch.timestamps() {
    ///     for t in ts {
    ///         println!("{}", t);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn timestamps(&self) -> Option<TimestampIterator> {
        // Check for waveform properties
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
            std::io::Read::read_exact(reader, &mut out[out_cursor..out_cursor + read_bytes])?;
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

/// Iterator over channel data chunks.
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

/// Iterator over timestamps.
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
    /// Get the length of this slice in elements.
    pub fn len(&self) -> usize {
        let bytes = match &self.data {
            ChannelData::Mmap(b) => b.len(),
            ChannelData::Owned(b) => b.len(),
        };
        bytes / self.dtype.itemsize()
    }

    /// Check if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this slice uses zero-copy memory mapping.
    pub fn is_zero_copy(&self) -> bool {
        matches!(self.data, ChannelData::Mmap(_))
    }

    /// Get typed access to the data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tdms::TdmsFile;
    ///
    /// let f = TdmsFile::open("data.tdms")?;
    /// let ch = f.group("G")?.channel("C")?;
    /// let slice = ch.read(0..100)?;
    /// let data: &[f64] = slice.as_typed()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_typed<T: Pod>(&self) -> Result<&[T]> {
        // Verify type matches
        if std::mem::size_of::<T>() != self.dtype.itemsize() {
            return Err(TdmsError::TypeMismatch);
        }

        let bytes = match &self.data {
            ChannelData::Mmap(b) => b,
            ChannelData::Owned(b) => b.as_slice(),
        };

        // Check alignment
        if bytes.as_ptr() as usize % std::mem::align_of::<T>() != 0 {
            return Err(TdmsError::AlignmentError);
        }

        // Safety: We've verified size and alignment
        let ptr = bytes.as_ptr() as *const T;
        let len = bytes.len() / std::mem::size_of::<T>();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    }
}

impl TdmsDType {
    /// Get the size in bytes of one element of this type.
    pub fn itemsize(&self) -> usize {
        match self {
            TdmsDType::I8 | TdmsDType::U8 | TdmsDType::Bool => 1,
            TdmsDType::I16 | TdmsDType::U16 => 2,
            TdmsDType::I32 | TdmsDType::U32 | TdmsDType::F32 => 4,
            TdmsDType::I64 | TdmsDType::U64 | TdmsDType::F64 => 8,
            TdmsDType::TimeStamp => 16,
            TdmsDType::String => 0, // Variable size
        }
    }
}

/// Marker trait for types that can be safely cast from byte slices.
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

fn dtype_from_internal(dt: &datatypes::DataType) -> TdmsDType {
    match dt {
        datatypes::DataType::I8 => TdmsDType::I8,
        datatypes::DataType::I16 => TdmsDType::I16,
        datatypes::DataType::I32 => TdmsDType::I32,
        datatypes::DataType::I64 => TdmsDType::I64,
        datatypes::DataType::U8 => TdmsDType::U8,
        datatypes::DataType::U16 => TdmsDType::U16,
        datatypes::DataType::U32 => TdmsDType::U32,
        datatypes::DataType::U64 => TdmsDType::U64,
        datatypes::DataType::SingleFloat => TdmsDType::F32,
        datatypes::DataType::DoubleFloat => TdmsDType::F64,
        datatypes::DataType::Boolean => TdmsDType::Bool,
        datatypes::DataType::String => TdmsDType::String,
        datatypes::DataType::TimeStamp => TdmsDType::TimeStamp,
        _ => TdmsDType::F64, // Fallback
    }
}
