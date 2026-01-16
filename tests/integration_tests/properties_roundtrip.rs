//! Tests round-trip of all property types at file, group, and channel levels.
//! Ensures PropertyValue encoding/decoding is correct for all supported variants.

use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

#[test]
fn file_properties_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/file_props.tdms";
    std::fs::create_dir_all("tests/output")?;

    {
        let mut writer = TdmsWriter::create(path)?;
        writer.add_property("i8", PropertyValue::I8(-8))?;
        writer.add_property("i16", PropertyValue::I16(-16000))?;
        writer.add_property("i32", PropertyValue::I32(-2000000))?;
        writer.add_property("i64", PropertyValue::I64(-8000000000000000000))?;
        writer.add_property("u8", PropertyValue::U8(200))?;
        writer.add_property("u16", PropertyValue::U16(50000))?;
        writer.add_property("u32", PropertyValue::U32(3000000000))?;
        writer.add_property("u64", PropertyValue::U64(17000000000000000000))?;
        writer.add_property("f32", PropertyValue::Float(1.2345))?;
        writer.add_property("f64", PropertyValue::Double(std::f64::consts::E))?;
        writer.add_property("bool", PropertyValue::Boolean(true))?;
        writer.add_property("string", PropertyValue::String("test".into()))?;
        writer.add_property("ts", PropertyValue::TimeStamp((1609459200, 123456789)))?;

        // Minimal channel to finalize file
        let mut group = writer.add_group("G")?;
        let mut ch = group.add_channel::<f64>("C")?;
        ch.write(&[0.0])?;
        writer.close()?;
    }

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

    assert_prop!("i8", I8, -8);
    assert_prop!("i16", I16, -16000);
    assert_prop!("i32", I32, -2000000);
    assert_prop!("i64", I64, -8000000000000000000);
    assert_prop!("u8", U8, 200);
    assert_prop!("u16", U16, 50000);
    assert_prop!("u32", U32, 3000000000);
    assert_prop!("u64", U64, 17000000000000000000);
    assert_prop!("f32", Float, 1.2345);
    assert_prop!("f64", Double, std::f64::consts::E);
    assert_prop!("bool", Boolean, true);
    assert_prop!("string", String, "test".to_string());
    assert_prop!("ts", TimeStamp, (1609459200, 123456789));

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn group_and_channel_properties_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/group_channel_props.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Group")?;
        group.add_property("g_i32", PropertyValue::I32(123))?;
        group.add_property("g_str", PropertyValue::String("group".into()))?;

        let mut channel = group.add_channel::<f64>("Channel")?;
        channel.add_property("c_f64", PropertyValue::Double(std::f64::consts::PI))?;
        channel.add_property("c_bool", PropertyValue::Boolean(false))?;
        channel.write(&[1.0, 2.0])?;
        writer.close()?;
    }

    let file = TdmsFile::open(path)?;
    let group = file.group("Group").unwrap();
    let channel = group.channel("Channel").unwrap();

    let group_props: std::collections::HashMap<_, _> = group
        .properties()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    let channel_props: std::collections::HashMap<_, _> = channel
        .properties()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();

    match group_props.get("g_i32") {
        Some(PropertyValue::I32(v)) => assert_eq!(*v, 123),
        _ => panic!("Missing or wrong g_i32"),
    }
    match group_props.get("g_str") {
        Some(PropertyValue::String(v)) => assert_eq!(v, "group"),
        _ => panic!("Missing or wrong g_str"),
    }

    match channel_props.get("c_f64") {
        Some(PropertyValue::Double(v)) => assert_eq!(*v, std::f64::consts::PI),
        _ => panic!("Missing or wrong c_f64"),
    }
    match channel_props.get("c_bool") {
        Some(PropertyValue::Boolean(v)) => assert!(!(*v)),
        _ => panic!("Missing or wrong c_bool"),
    }

    std::fs::remove_file(path)?;
    Ok(())
}
