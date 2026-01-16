//! Demonstrates setting all supported file-level property types.
//! No channel data is written; this focuses purely on metadata.

use std::path::Path;
use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("file_properties.tdms");

    // Write a file with all supported property types at the file level
    {
        let mut writer = TdmsWriter::create(path)?;

        // Integer properties
        writer.add_property("prop_i8", PropertyValue::I8(-8))?;
        writer.add_property("prop_i16", PropertyValue::I16(-16000))?;
        writer.add_property("prop_i32", PropertyValue::I32(-2000000000))?;
        writer.add_property("prop_i64", PropertyValue::I64(-9000000000000000000))?;

        // Unsigned integer properties
        writer.add_property("prop_u8", PropertyValue::U8(200))?;
        writer.add_property("prop_u16", PropertyValue::U16(50000))?;
        writer.add_property("prop_u32", PropertyValue::U32(3000000000))?;
        writer.add_property("prop_u64", PropertyValue::U64(18000000000000000000))?;

        // Float properties
        writer.add_property("prop_f32", PropertyValue::Float(1.2345))?;
        writer.add_property("prop_f64", PropertyValue::Double(std::f64::consts::E))?;

        // Boolean and string properties
        writer.add_property("prop_bool", PropertyValue::Boolean(true))?;
        writer.add_property("prop_string", PropertyValue::String("Hello TDMS".into()))?;

        // Timestamp property
        writer.add_property(
            "prop_timestamp",
            PropertyValue::TimeStamp((1700000000, 123456789)),
        )?;

        // Must write at least one channel to finalize the file
        let mut group = writer.add_group("Dummy")?;
        let mut channel = group.add_channel::<f64>("DummyChannel")?;
        channel.write(&[0.0])?;
        writer.close()?;
    }

    // Read back and verify all properties
    let file = TdmsFile::open(path)?;
    let props: std::collections::HashMap<_, _> = file
        .properties()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();

    macro_rules! assert_prop {
        ($key:expr, $variant:ident, $value:expr) => {
            match props.get($key) {
                Some(PropertyValue::$variant(v)) => assert_eq!(*v, $value),
                _ => panic!("Missing or wrong type for {}", $key),
            }
        };
    }

    assert_prop!("prop_i8", I8, -8);
    assert_prop!("prop_i16", I16, -16000);
    assert_prop!("prop_i32", I32, -2000000000);
    assert_prop!("prop_i64", I64, -9000000000000000000);
    assert_prop!("prop_u8", U8, 200);
    assert_prop!("prop_u16", U16, 50000);
    assert_prop!("prop_u32", U32, 3000000000);
    assert_prop!("prop_u64", U64, 18000000000000000000);
    assert_prop!("prop_f32", Float, 1.2345);
    assert_prop!("prop_f64", Double, std::f64::consts::E);
    assert_prop!("prop_bool", Boolean, true);
    assert_prop!("prop_string", String, "Hello TDMS".to_string());
    assert_prop!("prop_timestamp", TimeStamp, (1700000000, 123456789));

    println!("All {} file properties verified.", props.len());

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
