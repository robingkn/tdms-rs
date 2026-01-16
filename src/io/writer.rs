use crate::error::Result;
use crate::io::ext::TdmsWriteExt;
use std::io::Write;

pub struct TdmsIoWriter<W: Write> {
    inner: W,
}

impl<W: Write> TdmsIoWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { inner: writer }
    }

    pub fn write_string(&mut self, s: &str) -> Result<()> {
        self.inner.write_u32(s.len() as u32)?;
        self.inner.write_all(s.as_bytes())?;
        Ok(())
    }

    pub fn write_timestamp(&mut self, seconds: i64, fraction: u64) -> Result<()> {
        self.inner.write_u64(fraction)?;
        self.inner.write_i64(seconds)?;
        Ok(())
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}
