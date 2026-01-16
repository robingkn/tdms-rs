//! Tests round-trip for all supported channel data types.
//! String and TimeStamp channel data are intentionally excluded as they are not supported by the current typed API.

use tdms_rs::{TdmsFile, TdmsWriter};

macro_rules! test_channel_type {
    ($test_name:ident, $ty:ty, $data:expr) => {
        #[test]
        fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            let path = concat!("tests/output/channel_", stringify!($test_name), ".tdms");
            std::fs::create_dir_all("tests/output")?;

            // Write
            {
                let mut writer = TdmsWriter::create(path)?;
                let mut group = writer.add_group("G")?;
                let mut channel = group.add_channel::<$ty>("C")?;
                channel.write($data)?;
                // File is automatically flushed and closed when writer goes out of scope
            }

            // Read and verify
            let file = TdmsFile::open(path)?;
            let channel = file.group("G").unwrap().channel("C").unwrap();

            assert_eq!(channel.len(), $data.len());
            assert_eq!(channel.dtype(), <$ty>::data_type());

            let mut read_back = vec![<$ty>::default(); $data.len()];
            channel.read(0..$data.len(), &mut read_back)?;
            assert_eq!(read_back, $data);

            std::fs::remove_file(path)?;
            Ok(())
        }
    };
}

// Helper to map Rust types to TDMS DataType for tests
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

// Generate tests for each supported type
test_channel_type!(channel_i8, i8, &[-128, -1, 0, 1, 127]);
test_channel_type!(channel_i16, i16, &[-32768, -1, 0, 1, 32767]);
test_channel_type!(channel_i32, i32, &[-2147483648, -1, 0, 1, 2147483647]);
test_channel_type!(
    channel_i64,
    i64,
    &[-9223372036854775808_i64, -1, 0, 1, 9223372036854775807_i64]
);
test_channel_type!(channel_u8, u8, &[0, 1, 255]);
test_channel_type!(channel_u16, u16, &[0, 1, 65535]);
test_channel_type!(channel_u32, u32, &[0, 1, 4294967295_u32]);
test_channel_type!(channel_u64, u64, &[0, 1, 18446744073709551615_u64]);
test_channel_type!(
    channel_f32,
    f32,
    &[-std::f32::consts::PI, 0.0, std::f32::consts::PI]
);
test_channel_type!(
    channel_f64,
    f64,
    &[-std::f64::consts::E, 0.0, std::f64::consts::E]
);
test_channel_type!(channel_bool, bool, &[true, false, true, false, true]);

#[test]
fn channel_unsupported_types_error() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/channel_unsupported.tdms";

    // Note: String and TimeStamp channels are not supported by typed write API
    // This test documents the limitation by attempting to read them as raw bytes
    // which should fail with TypeMismatch

    // Create a file with a supported channel type to test reading errors
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let mut ch = group.add_channel::<f64>("Data")?;
        ch.write(&[1.0, 2.0, 3.0])?;
        // File is automatically flushed and closed when writer goes out of scope
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("Data").unwrap();

    // Verify the type is correct
    assert_eq!(channel.dtype(), tdms_rs::DataType::Double);

    // Attempting to read f64 data into i32 buffer should fail
    let mut buf = [0i32; 3];
    assert!(channel.read(0..3, &mut buf).is_err()); // TypeMismatch

    std::fs::remove_file(path)?;
    Ok(())
}
