use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub trait TdmsReadExt: Read {
    fn read_u8(&mut self) -> std::io::Result<u8> {
        ReadBytesExt::read_u8(self)
    }

    fn read_i8(&mut self) -> std::io::Result<i8> {
        ReadBytesExt::read_i8(self)
    }

    fn read_u16(&mut self) -> std::io::Result<u16> {
        ReadBytesExt::read_u16::<LittleEndian>(self)
    }

    fn read_i16(&mut self) -> std::io::Result<i16> {
        ReadBytesExt::read_i16::<LittleEndian>(self)
    }

    fn read_u32(&mut self) -> std::io::Result<u32> {
        ReadBytesExt::read_u32::<LittleEndian>(self)
    }

    fn read_i32(&mut self) -> std::io::Result<i32> {
        ReadBytesExt::read_i32::<LittleEndian>(self)
    }

    fn read_u64(&mut self) -> std::io::Result<u64> {
        ReadBytesExt::read_u64::<LittleEndian>(self)
    }

    fn read_i64(&mut self) -> std::io::Result<i64> {
        ReadBytesExt::read_i64::<LittleEndian>(self)
    }

    fn read_f32(&mut self) -> std::io::Result<f32> {
        ReadBytesExt::read_f32::<LittleEndian>(self)
    }

    fn read_f64(&mut self) -> std::io::Result<f64> {
        ReadBytesExt::read_f64::<LittleEndian>(self)
    }
}

impl<R: Read + ?Sized> TdmsReadExt for R {}

pub trait TdmsWriteExt: Write {
    fn write_u8(&mut self, n: u8) -> std::io::Result<()> {
        WriteBytesExt::write_u8(self, n)
    }

    fn write_i8(&mut self, n: i8) -> std::io::Result<()> {
        WriteBytesExt::write_i8(self, n)
    }

    fn write_u16(&mut self, n: u16) -> std::io::Result<()> {
        WriteBytesExt::write_u16::<LittleEndian>(self, n)
    }

    fn write_i16(&mut self, n: i16) -> std::io::Result<()> {
        WriteBytesExt::write_i16::<LittleEndian>(self, n)
    }

    fn write_u32(&mut self, n: u32) -> std::io::Result<()> {
        WriteBytesExt::write_u32::<LittleEndian>(self, n)
    }

    fn write_i32(&mut self, n: i32) -> std::io::Result<()> {
        WriteBytesExt::write_i32::<LittleEndian>(self, n)
    }

    fn write_u64(&mut self, n: u64) -> std::io::Result<()> {
        WriteBytesExt::write_u64::<LittleEndian>(self, n)
    }

    fn write_i64(&mut self, n: i64) -> std::io::Result<()> {
        WriteBytesExt::write_i64::<LittleEndian>(self, n)
    }

    fn write_f32(&mut self, n: f32) -> std::io::Result<()> {
        WriteBytesExt::write_f32::<LittleEndian>(self, n)
    }

    fn write_f64(&mut self, n: f64) -> std::io::Result<()> {
        WriteBytesExt::write_f64::<LittleEndian>(self, n)
    }
}

impl<W: Write + ?Sized> TdmsWriteExt for W {}
