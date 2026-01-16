//! Demonstrates writing and reading all supported channel data types.
//! String and TimeStamp channel data are not supported by the current typed API
//! and are intentionally excluded.

use std::path::Path;
use tdms_rs::{TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("channel_all_types.tdms");

    // Write all supported channel data types
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("DataTypes")?;

        // Integer types
        let mut ch_i8 = group.add_channel::<i8>("Int8")?;
        ch_i8.write(&[-128, -1, 0, 1, 127])?;

        let mut ch_i16 = group.add_channel::<i16>("Int16")?;
        ch_i16.write(&[-32768, -1, 0, 1, 32767])?;

        let mut ch_i32 = group.add_channel::<i32>("Int32")?;
        ch_i32.write(&[-2147483648, -1, 0, 1, 2147483647])?;

        let mut ch_i64 = group.add_channel::<i64>("Int64")?;
        ch_i64.write(&[-9223372036854775808, -1, 0, 1, 9223372036854775807])?;

        // Unsigned integer types
        let mut ch_u8 = group.add_channel::<u8>("Uint8")?;
        ch_u8.write(&[0, 1, 255])?;

        let mut ch_u16 = group.add_channel::<u16>("Uint16")?;
        ch_u16.write(&[0, 1, 65535])?;

        let mut ch_u32 = group.add_channel::<u32>("Uint32")?;
        ch_u32.write(&[0, 1, 4294967295])?;

        let mut ch_u64 = group.add_channel::<u64>("Uint64")?;
        ch_u64.write(&[0, 1, 18446744073709551615])?;

        // Float types
        let mut ch_f32 = group.add_channel::<f32>("Float32")?;
        ch_f32.write(&[-std::f32::consts::PI, 0.0, std::f32::consts::PI])?;

        let mut ch_f64 = group.add_channel::<f64>("Float64")?;
        ch_f64.write(&[-std::f64::consts::E, 0.0, std::f64::consts::E])?;

        // Boolean type
        let mut ch_bool = group.add_channel::<bool>("Boolean")?;
        ch_bool.write(&[true, false, true, false, true])?;

        // File is automatically flushed and closed when writer goes out of scope
    }

    // Read back and verify all channel data
    let file = TdmsFile::open(path)?;
    let group = file.group("DataTypes").unwrap();

    macro_rules! verify_channel {
        ($group:expr, $name:expr, $ty:ty, $expected:expr) => {
            let ch = $group.channel($name).unwrap();
            assert_eq!(ch.dtype(), <$ty>::data_type());
            let mut data = vec![<$ty>::default(); ch.len()];
            ch.read(0..ch.len(), &mut data).unwrap();
            assert_eq!(data, $expected);
            println!("{} channel verified: {} values", $name, data.len());
        };
    }

    verify_channel!(group, "Int8", i8, &[-128, -1, 0, 1, 127]);
    verify_channel!(group, "Int16", i16, &[-32768, -1, 0, 1, 32767]);
    verify_channel!(group, "Int32", i32, &[-2147483648, -1, 0, 1, 2147483647]);
    verify_channel!(
        group,
        "Int64",
        i64,
        &[-9223372036854775808, -1, 0, 1, 9223372036854775807]
    );
    verify_channel!(group, "Uint8", u8, &[0, 1, 255]);
    verify_channel!(group, "Uint16", u16, &[0, 1, 65535]);
    verify_channel!(group, "Uint32", u32, &[0, 1, 4294967295]);
    verify_channel!(group, "Uint64", u64, &[0, 1, 18446744073709551615]);
    verify_channel!(
        group,
        "Float32",
        f32,
        &[-std::f32::consts::PI, 0.0, std::f32::consts::PI]
    );
    verify_channel!(
        group,
        "Float64",
        f64,
        &[-std::f64::consts::E, 0.0, std::f64::consts::E]
    );
    verify_channel!(group, "Boolean", bool, &[true, false, true, false, true]);

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}

// Helper to map Rust types to TDMS DataType for the macro above
trait DataTypeExt {
    fn data_type() -> tdms_rs::DataType;
}
impl DataTypeExt for i8 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::I8
    }
}
impl DataTypeExt for i16 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::I16
    }
}
impl DataTypeExt for i32 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::I32
    }
}
impl DataTypeExt for i64 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::I64
    }
}
impl DataTypeExt for u8 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::U8
    }
}
impl DataTypeExt for u16 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::U16
    }
}
impl DataTypeExt for u32 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::U32
    }
}
impl DataTypeExt for u64 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::U64
    }
}
impl DataTypeExt for f32 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::Float
    }
}
impl DataTypeExt for f64 {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::Double
    }
}
impl DataTypeExt for bool {
    fn data_type() -> tdms_rs::DataType {
        tdms_rs::DataType::Boolean
    }
}
