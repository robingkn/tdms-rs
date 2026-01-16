use crate::error::{Result, TdmsError};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    I8 = 1,
    I16 = 2,
    I32 = 3,
    I64 = 4,
    U8 = 5,
    U16 = 6,
    U32 = 7,
    U64 = 8,
    Float = 9,
    Double = 10,
    String = 32,
    Boolean = 33,
    TimeStamp = 68,
}

impl DataType {
    pub fn from_u32(val: u32) -> Result<Self> {
        match val {
            1 => Ok(DataType::I8),
            2 => Ok(DataType::I16),
            3 => Ok(DataType::I32),
            4 => Ok(DataType::I64),
            5 => Ok(DataType::U8),
            6 => Ok(DataType::U16),
            7 => Ok(DataType::U32),
            8 => Ok(DataType::U64),
            9 => Ok(DataType::Float),
            10 => Ok(DataType::Double),
            32 => Ok(DataType::String),
            33 => Ok(DataType::Boolean),
            68 => Ok(DataType::TimeStamp),
            0 => Err(TdmsError::NotImplemented(
                "Void DataType not supported in public API".to_string(),
            )),
            _ => Err(TdmsError::NotImplemented(format!("DataType {}", val))),
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            DataType::I8 => 1,
            DataType::I16 => 2,
            DataType::I32 => 3,
            DataType::I64 => 4,
            DataType::U8 => 5,
            DataType::U16 => 6,
            DataType::U32 => 7,
            DataType::U64 => 8,
            DataType::Float => 9,
            DataType::Double => 10,
            DataType::String => 32,
            DataType::Boolean => 33,
            DataType::TimeStamp => 68,
        }
    }

    pub fn itemsize(&self) -> usize {
        match self {
            DataType::I8 | DataType::U8 | DataType::Boolean => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::Float => 4,
            DataType::I64 | DataType::U64 | DataType::Double => 8,
            DataType::TimeStamp => 16,
            DataType::String => 0,
        }
    }
}

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
    TimeStamp((i64, u64)),
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PropertyValue::String(s) => write!(f, "\"{}\"", s),
            PropertyValue::Double(d) => write!(f, "{:.6}", d),
            PropertyValue::Float(fl) => write!(f, "{:.6}", fl),
            PropertyValue::I8(i) => write!(f, "{}", i),
            PropertyValue::I16(i) => write!(f, "{}", i),
            PropertyValue::I32(i) => write!(f, "{}", i),
            PropertyValue::I64(i) => write!(f, "{}", i),
            PropertyValue::U8(u) => write!(f, "{}", u),
            PropertyValue::U16(u) => write!(f, "{}", u),
            PropertyValue::U32(u) => write!(f, "{}", u),
            PropertyValue::U64(u) => write!(f, "{}", u),
            PropertyValue::Boolean(b) => write!(f, "{}", b),
            PropertyValue::TimeStamp((s, frac)) => write!(f, "{}.{:019}", s, frac),
        }
    }
}

impl From<(i64, u64)> for PropertyValue {
    fn from((seconds, fraction): (i64, u64)) -> Self {
        PropertyValue::TimeStamp((seconds, fraction))
    }
}

// Additional From implementations for common literal types
impl From<&String> for PropertyValue {
    fn from(s: &String) -> Self {
        PropertyValue::String(s.clone())
    }
}

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

impl From<f64> for PropertyValue {
    fn from(f: f64) -> Self {
        PropertyValue::Double(f)
    }
}

impl From<f32> for PropertyValue {
    fn from(f: f32) -> Self {
        PropertyValue::Float(f)
    }
}

impl From<i32> for PropertyValue {
    fn from(i: i32) -> Self {
        PropertyValue::I32(i)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Boolean(b)
    }
}
