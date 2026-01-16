use crate::error::{Result, TdmsError};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};

pub struct TdmsIoReader<R: Read + Seek> {
    inner: R,
}

impl<R: Read + Seek> TdmsIoReader<R> {
    pub fn new(reader: R) -> Self {
        Self { inner: reader }
    }

    pub fn read_string(&mut self) -> Result<String> {
        let len = self.inner.read_u32::<LittleEndian>()?;
        let mut buf = vec![0u8; len as usize];
        self.inner.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|_| TdmsError::StringEncoding)
    }

    pub fn read_timestamp(&mut self) -> Result<(i64, u64)> {
        let fraction = self.inner.read_u64::<LittleEndian>()?;
        let seconds = self.inner.read_i64::<LittleEndian>()?;
        Ok((seconds, fraction))
    }

    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}
