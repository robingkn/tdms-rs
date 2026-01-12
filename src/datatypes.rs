//! TDMS data types and property values.
//!
//! This module defines the core data types used in TDMS files:
//! - [`TdmsData`] - Enum representing channel data of various types
//! - [`PropertyValue`] - Enum representing property metadata values
//! - [`DataType`] - Internal enum for TDMS type codes

use crate::error::{Result, TdmsError};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt::{Display, Formatter};
use std::io::{Read, Seek};

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Void = 0,
    I8 = 1,
    I16 = 2,
    I32 = 3,
    I64 = 4,
    U8 = 5,
    U16 = 6,
    U32 = 7,
    U64 = 8,
    SingleFloat = 9,
    DoubleFloat = 10,
    String = 32,
    Boolean = 33,
    TimeStamp = 68,
    // extended types later
}

/// Property values stored as metadata in TDMS files.
///
/// Properties provide metadata about files, groups, and channels. They are stored
/// as key-value pairs where the key is a string and the value is one of the
/// supported TDMS data types.
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::{TdmsFile, PropertyValue};
/// use std::path::Path;
///
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
///
/// if let Some(group) = file.groups.get("Sensors") {
///     for (prop_name, prop_value) in &group.properties {
///         match prop_value {
///             PropertyValue::String(s) => println!("String property {}: {}", prop_name, s),
///             PropertyValue::Double(d) => println!("Numeric property {}: {}", prop_name, d),
///             PropertyValue::Boolean(b) => println!("Boolean property {}: {}", prop_name, b),
///             _ => println!("Other property {}: {:?}", prop_name, prop_value),
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq)]

pub enum PropertyValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Float(f32),
    Double(f64),
    String(String),
    Boolean(bool),
    /// TDMS timestamp as (seconds since 1904-01-01 UTC, fraction in 2^-64 second units)
    TimeStamp((i64, u64)),
}

/// Channel data stored in TDMS files.
///
/// This enum represents the actual measurement data contained in TDMS channels.
/// Each variant corresponds to a specific TDMS data type and contains a vector
/// of values of that type.
///
/// # Data Type Mapping
///
/// | TDMS Type | Rust Type | Description |
/// |-----------|-----------|-------------|
/// | I8        | `i8`      | 8-bit signed integer |
/// | I16       | `i16`     | 16-bit signed integer |
/// | I32       | `i32`     | 32-bit signed integer |
/// | I64       | `i64`     | 64-bit signed integer |
/// | U8        | `u8`      | 8-bit unsigned integer |
/// | U16       | `u16`     | 16-bit unsigned integer |
/// | U32       | `u32`     | 32-bit unsigned integer |
/// | U64       | `u64`     | 64-bit unsigned integer |
/// | Float     | `f32`     | 32-bit floating point |
/// | Double    | `f64`     | 64-bit floating point |
/// | String    | `String`  | UTF-8 encoded text |
/// | Boolean   | `bool`    | True/false values |
/// | TimeStamp | `(i64, u64)` | TDMS timestamp (seconds since 1904, fraction) |
///
/// # Examples
///
/// ```no_run
/// use tdms_rs::{TdmsFile, TdmsData};
/// use std::path::Path;
///
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
///
/// if let Some(group) = file.groups.get("Measurements") {
///     if let Some(channel) = group.channels.get("Voltage") {
///         match &channel.data {
///             Some(TdmsData::Double(values)) => {
///                 println!("Voltage readings: {} samples", values.len());
///                 let avg = values.iter().sum::<f64>() / values.len() as f64;
///                 println!("Average voltage: {:.3} V", avg);
///             },
///             Some(TdmsData::Float(values)) => {
///                 println!("Float voltage readings: {} samples", values.len());
///             },
///             Some(TdmsData::TimeStamp(timestamps)) => {
///                 println!("Timestamp data: {} entries", timestamps.len());
///                 for (seconds, fraction) in timestamps.iter().take(3) {
///                     println!("  Time: {} seconds + {} fraction", seconds, fraction);
///                 }
///             },
///             Some(other) => println!("Unexpected data type: {:?}", other),
///             None => println!("No data in channel"),
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Timestamp Format
///
/// TDMS timestamps are represented as `(i64, u64)` tuples where:
/// - The `i64` value is seconds since January 1, 1904, 00:00:00 UTC
/// - The `u64` value is the fractional part in units of 2^-64 seconds
///
/// This provides very high precision timing information suitable for
/// high-frequency data acquisition applications.
#[derive(Debug, PartialEq, Clone)]
pub enum TdmsData {
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    String(Vec<String>),
    Boolean(Vec<bool>),
    TimeStamp(Vec<(i64, u64)>),
}

impl TdmsData {
    /// Get the number of elements in this data.
    pub fn len(&self) -> usize {
        match self {
            TdmsData::I8(v) => v.len(),
            TdmsData::I16(v) => v.len(),
            TdmsData::I32(v) => v.len(),
            TdmsData::I64(v) => v.len(),
            TdmsData::U8(v) => v.len(),
            TdmsData::U16(v) => v.len(),
            TdmsData::U32(v) => v.len(),
            TdmsData::U64(v) => v.len(),
            TdmsData::Float(v) => v.len(),
            TdmsData::Double(v) => v.len(),
            TdmsData::String(v) => v.len(),
            TdmsData::Boolean(v) => v.len(),
            TdmsData::TimeStamp(v) => v.len(),
        }
    }

    /// Check if this data is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a human-readable name for this data type.
    pub fn type_name(&self) -> &'static str {
        match self {
            TdmsData::I8(_) => "I8",
            TdmsData::I16(_) => "I16",
            TdmsData::I32(_) => "I32",
            TdmsData::I64(_) => "I64",
            TdmsData::U8(_) => "U8",
            TdmsData::U16(_) => "U16",
            TdmsData::U32(_) => "U32",
            TdmsData::U64(_) => "U64",
            TdmsData::Float(_) => "Float",
            TdmsData::Double(_) => "Double",
            TdmsData::String(_) => "String",
            TdmsData::Boolean(_) => "Boolean",
            TdmsData::TimeStamp(_) => "TimeStamp",
        }
    }

    /// Check if this data type is numeric.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            TdmsData::I8(_)
                | TdmsData::I16(_)
                | TdmsData::I32(_)
                | TdmsData::I64(_)
                | TdmsData::U8(_)
                | TdmsData::U16(_)
                | TdmsData::U32(_)
                | TdmsData::U64(_)
                | TdmsData::Float(_)
                | TdmsData::Double(_)
        )
    }

    /// Extend this data with additional data of the same type.
    ///
    /// This method is used internally when reading multi-segment TDMS files
    /// where channel data may be split across multiple segments.
    ///
    /// # Arguments
    ///
    /// * `other` - Additional data to append to this data
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the data was successfully extended, or an error
    /// if the data types don't match.
    ///
    /// # Errors
    ///
    /// Returns an error if the two `TdmsData` variants don't match
    /// (e.g., trying to extend `Double` data with `I32` data).
    pub fn extend(&mut self, other: TdmsData) -> Result<()> {
        match (self, other) {
            (TdmsData::Double(v1), TdmsData::Double(v2)) => v1.extend(v2),
            (TdmsData::I32(v1), TdmsData::I32(v2)) => v1.extend(v2),
            (TdmsData::String(v1), TdmsData::String(v2)) => v1.extend(v2),
            (TdmsData::Boolean(v1), TdmsData::Boolean(v2)) => v1.extend(v2),
            (TdmsData::I8(v1), TdmsData::I8(v2)) => v1.extend(v2),
            (TdmsData::I16(v1), TdmsData::I16(v2)) => v1.extend(v2),
            (TdmsData::I64(v1), TdmsData::I64(v2)) => v1.extend(v2),
            (TdmsData::U8(v1), TdmsData::U8(v2)) => v1.extend(v2),
            (TdmsData::U16(v1), TdmsData::U16(v2)) => v1.extend(v2),
            (TdmsData::U32(v1), TdmsData::U32(v2)) => v1.extend(v2),
            (TdmsData::U64(v1), TdmsData::U64(v2)) => v1.extend(v2),
            (TdmsData::Float(v1), TdmsData::Float(v2)) => v1.extend(v2),
            (TdmsData::TimeStamp(v1), TdmsData::TimeStamp(v2)) => v1.extend(v2),
            _ => {
                return Err(TdmsError::NotImplemented(
                    "Data type mismatch during extension".to_string(),
                ))
            }
        }
        Ok(())
    }
}

impl DataType {
    pub fn from_u32(val: u32) -> Result<Self> {
        match val {
            0 => Ok(DataType::Void),
            1 => Ok(DataType::I8),
            2 => Ok(DataType::I16),
            3 => Ok(DataType::I32),
            4 => Ok(DataType::I64),
            5 => Ok(DataType::U8),
            6 => Ok(DataType::U16),
            7 => Ok(DataType::U32),
            8 => Ok(DataType::U64),
            9 => Ok(DataType::SingleFloat),
            10 => Ok(DataType::DoubleFloat),
            32 => Ok(DataType::String),
            33 => Ok(DataType::Boolean),
            68 => Ok(DataType::TimeStamp),
            _ => Err(TdmsError::NotImplemented(format!("DataType {}", val))),
        }
    }
}

pub fn read_property_value<R: Read + Seek>(
    reader: &mut R,
    type_code: u32,
) -> Result<PropertyValue> {
    let dtype = DataType::from_u32(type_code)?;

    match dtype {
        DataType::I8 => Ok(PropertyValue::I8(reader.read_i8()?)),
        DataType::I16 => Ok(PropertyValue::I16(reader.read_i16::<LittleEndian>()?)),
        DataType::I32 => Ok(PropertyValue::I32(reader.read_i32::<LittleEndian>()?)),
        DataType::I64 => Ok(PropertyValue::I64(reader.read_i64::<LittleEndian>()?)),
        DataType::U8 => Ok(PropertyValue::U8(reader.read_u8()?)),
        DataType::U16 => Ok(PropertyValue::U16(reader.read_u16::<LittleEndian>()?)),
        DataType::U32 => Ok(PropertyValue::U32(reader.read_u32::<LittleEndian>()?)),
        DataType::U64 => Ok(PropertyValue::U64(reader.read_u64::<LittleEndian>()?)),
        DataType::SingleFloat => Ok(PropertyValue::Float(reader.read_f32::<LittleEndian>()?)),
        DataType::DoubleFloat => Ok(PropertyValue::Double(reader.read_f64::<LittleEndian>()?)),
        DataType::Boolean => Ok(PropertyValue::Boolean(reader.read_u8()? != 0)),
        DataType::String => {
            let len = reader.read_u32::<LittleEndian>()?;
            let mut buf = vec![0u8; len as usize];
            reader.read_exact(&mut buf)?;
            let s = String::from_utf8(buf).map_err(|_| TdmsError::StringEncoding)?;
            Ok(PropertyValue::String(s))
        }
        DataType::TimeStamp => {
            // TDMS timestamps: 8 bytes fraction (u64) + 8 bytes seconds (i64)
            // Fraction represents 2^-64 second precision
            // Seconds are since 1904-01-01 00:00:00 UTC
            let fraction = reader.read_u64::<LittleEndian>()?;
            let seconds = reader.read_i64::<LittleEndian>()?;
            Ok(PropertyValue::TimeStamp((seconds, fraction)))
        }
        _ => Err(TdmsError::NotImplemented(format!(
            "Reading prop type {:?}",
            dtype
        ))),
    }
}

pub fn read_raw_data<R: Read + Seek>(
    reader: &mut R,
    data_type: &DataType,
    count: u64,
    total_size_bytes: Option<u64>,
) -> Result<TdmsData> {
    let count = count as usize;
    match data_type {
        DataType::Void => Err(TdmsError::NotImplemented("Raw data for Void".to_string())),
        DataType::String => {
            // String Data Parsing
            let total_size = total_size_bytes
                .ok_or(TdmsError::NotImplemented("String size missing".to_string()))?;
            let offsets_size = (count * 4) as u64;

            let mut offsets = Vec::with_capacity(count);
            for _ in 0..count {
                let offset = reader.read_u32::<LittleEndian>()?;
                offsets.push(offset);
            }


            // Calculate Char Size
            // Offsets are relative to the start of the character data.
            // TotalSize in meta (45) INCLUDES offsets (20).
            // Char data size = 25.
            // Offsets: 5, 10, 10, 14, 25.
            // These are End Offsets.
            // Final offset (25) matches Char Size.

            let char_size = total_size.saturating_sub(offsets_size);

            let mut data_bytes = vec![0u8; char_size as usize];
            reader.read_exact(&mut data_bytes)?;

            let mut strings = Vec::with_capacity(count);
            let mut start = 0;
            for (_i, &offset) in offsets.iter().enumerate().take(count) {
                let end = offset as usize;

                // Bounds check
                if end > data_bytes.len() {
                    // Truncated string
                    if start < data_bytes.len() {
                        let slice = &data_bytes[start..];
                        strings.push(String::from_utf8_lossy(slice).into_owned());
                    } else {
                        strings.push(String::new());
                    }
                } else if start <= end {
                    let slice = &data_bytes[start..end];
                    strings.push(String::from_utf8_lossy(slice).into_owned());
                } else {
                    strings.push(String::new());
                }
                start = end;
            }
            Ok(TdmsData::String(strings))
        }
        DataType::I8 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_i8()?);
            }
            Ok(TdmsData::I8(data))
        }
        DataType::I16 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_i16::<LittleEndian>()?);
            }
            Ok(TdmsData::I16(data))
        }
        DataType::I32 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_i32::<LittleEndian>()?);
            }
            Ok(TdmsData::I32(data))
        }
        DataType::I64 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_i64::<LittleEndian>()?);
            }
            Ok(TdmsData::I64(data))
        }
        DataType::U8 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_u8()?);
            }
            Ok(TdmsData::U8(data))
        }
        DataType::U16 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_u16::<LittleEndian>()?);
            }
            Ok(TdmsData::U16(data))
        }
        DataType::U32 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_u32::<LittleEndian>()?);
            }
            Ok(TdmsData::U32(data))
        }
        DataType::U64 => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_u64::<LittleEndian>()?);
            }
            Ok(TdmsData::U64(data))
        }
        DataType::SingleFloat => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_f32::<LittleEndian>()?);
            }
            Ok(TdmsData::Float(data))
        }
        DataType::DoubleFloat => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_f64::<LittleEndian>()?);
            }
            Ok(TdmsData::Double(data))
        }
        DataType::Boolean => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                data.push(reader.read_u8()? != 0);
            }
            Ok(TdmsData::Boolean(data))
        }
        DataType::TimeStamp => {
            let mut data = Vec::with_capacity(count);
            for _ in 0..count {
                let fraction = reader.read_u64::<LittleEndian>()?;
                let seconds = reader.read_i64::<LittleEndian>()?;
                data.push((seconds, fraction));
            }
            Ok(TdmsData::TimeStamp(data))
        }
    }
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PropertyValue::String(s) => write!(f, "\"{}\"", s),
            PropertyValue::Double(d) => {
                if d.is_nan() {
                    write!(f, "NaN")
                } else if d.is_infinite() {
                    write!(f, "{}", if *d > 0.0 { "∞" } else { "-∞" })
                } else {
                    write!(f, "{:.6}", d)
                }
            }
            PropertyValue::Float(fl) => {
                if fl.is_nan() {
                    write!(f, "NaN")
                } else if fl.is_infinite() {
                    write!(f, "{}", if *fl > 0.0 { "∞" } else { "-∞" })
                } else {
                    write!(f, "{:.6}", fl)
                }
            }
            PropertyValue::I8(i) => write!(f, "{}", i),
            PropertyValue::I16(i) => write!(f, "{}", i),
            PropertyValue::I32(i) => write!(f, "{}", i),
            PropertyValue::I64(i) => write!(f, "{}", i),
            PropertyValue::U8(u) => write!(f, "{}", u),
            PropertyValue::U16(u) => write!(f, "{}", u),
            PropertyValue::U32(u) => write!(f, "{}", u),
            PropertyValue::U64(u) => write!(f, "{}", u),
            PropertyValue::Boolean(b) => write!(f, "{}", b),
            PropertyValue::TimeStamp((s, frac)) => {
                write!(f, "{}.{:019}", s, frac)
            }
        }
    }
}

impl Display for TdmsData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} [{}]", self.type_name(), self.len())
    }
}

// From implementations for PropertyValue to reduce boilerplate
impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(s.to_string())
    }
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self {
        PropertyValue::String(s)
    }
}

impl From<i8> for PropertyValue {
    fn from(i: i8) -> Self {
        PropertyValue::I8(i)
    }
}

impl From<i16> for PropertyValue {
    fn from(i: i16) -> Self {
        PropertyValue::I16(i)
    }
}

impl From<i32> for PropertyValue {
    fn from(i: i32) -> Self {
        PropertyValue::I32(i)
    }
}

impl From<i64> for PropertyValue {
    fn from(i: i64) -> Self {
        PropertyValue::I64(i)
    }
}

impl From<u8> for PropertyValue {
    fn from(u: u8) -> Self {
        PropertyValue::U8(u)
    }
}

impl From<u16> for PropertyValue {
    fn from(u: u16) -> Self {
        PropertyValue::U16(u)
    }
}

impl From<u32> for PropertyValue {
    fn from(u: u32) -> Self {
        PropertyValue::U32(u)
    }
}

impl From<u64> for PropertyValue {
    fn from(u: u64) -> Self {
        PropertyValue::U64(u)
    }
}

impl From<f32> for PropertyValue {
    fn from(f: f32) -> Self {
        PropertyValue::Float(f)
    }
}

impl From<f64> for PropertyValue {
    fn from(f: f64) -> Self {
        PropertyValue::Double(f)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Boolean(b)
    }
}

impl From<(i64, u64)> for PropertyValue {
    fn from((seconds, fraction): (i64, u64)) -> Self {
        PropertyValue::TimeStamp((seconds, fraction))
    }
}

pub fn create_empty_data(dtype: &DataType) -> Result<TdmsData> {
    match dtype {
        DataType::I8 => Ok(TdmsData::I8(Vec::new())),
        DataType::I16 => Ok(TdmsData::I16(Vec::new())),
        DataType::I32 => Ok(TdmsData::I32(Vec::new())),
        DataType::I64 => Ok(TdmsData::I64(Vec::new())),
        DataType::U8 => Ok(TdmsData::U8(Vec::new())),
        DataType::U16 => Ok(TdmsData::U16(Vec::new())),
        DataType::U32 => Ok(TdmsData::U32(Vec::new())),
        DataType::U64 => Ok(TdmsData::U64(Vec::new())),
        DataType::SingleFloat => Ok(TdmsData::Float(Vec::new())),
        DataType::DoubleFloat => Ok(TdmsData::Double(Vec::new())),
        DataType::Boolean => Ok(TdmsData::Boolean(Vec::new())),
        DataType::String => Ok(TdmsData::String(Vec::new())),
        DataType::TimeStamp => Ok(TdmsData::TimeStamp(Vec::new())),
        _ => Err(TdmsError::NotImplemented(format!(
            "Empty data for {:?}",
            dtype
        ))),
    }
}
