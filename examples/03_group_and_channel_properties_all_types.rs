//! Demonstrates setting all supported property types at group and channel levels.
//! Writes minimal channel data to finalize the file.

use std::path::Path;
use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("group_channel_properties.tdms");

    {
        let mut writer = TdmsWriter::create(path)?;

        // Group with all property types
        let mut group = writer.add_group("Sensors")?;
        group.add_property("group_i8", PropertyValue::I8(-8))?;
        group.add_property("group_u16", PropertyValue::U16(50000))?;
        group.add_property("group_f32", PropertyValue::Float(std::f32::consts::PI))?;
        group.add_property("group_bool", PropertyValue::Boolean(false))?;
        group.add_property("group_string", PropertyValue::String("GroupLevel".into()))?;
        group.add_property("group_ts", PropertyValue::TimeStamp((1609459200, 0)))?; // 2021-01-01

        // Channel with all property types
        let mut channel = group.add_channel::<f64>("Temperature")?;
        channel.add_property("ch_i32", PropertyValue::I32(123456))?;
        channel.add_property("ch_u64", PropertyValue::U64(9876543210))?;
        channel.add_property("ch_f64", PropertyValue::Double(std::f64::consts::E))?;
        channel.add_property("ch_bool", PropertyValue::Boolean(true))?;
        channel.add_property("ch_string", PropertyValue::String("ChannelLevel".into()))?;
        channel.add_property("ch_ts", PropertyValue::TimeStamp((1640995200, 500000000)))?; // 2022-01-01 + fraction

        // Write minimal data to finalize
        channel.write(&[20.0, 21.0, 22.0])?;
        writer.close()?;
    }

    // Read back and verify group and channel properties
    let file = TdmsFile::open(path)?;
    let group = file.group("Sensors").unwrap();
    let channel = group.channel("Temperature").unwrap();

    macro_rules! assert_prop {
        ($props:expr, $key:expr, $variant:ident, $value:expr) => {
            match $props.get($key) {
                Some(PropertyValue::$variant(v)) => assert_eq!(*v, $value),
                _ => panic!("Missing or wrong type for {}", $key),
            }
        };
    }

    let group_props: std::collections::HashMap<_, _> = group
        .properties()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    assert_prop!(group_props, "group_i8", I8, -8);
    assert_prop!(group_props, "group_u16", U16, 50000);
    assert_prop!(group_props, "group_f32", Float, std::f32::consts::PI);
    assert_prop!(group_props, "group_bool", Boolean, false);
    assert_prop!(
        group_props,
        "group_string",
        String,
        "GroupLevel".to_string()
    );
    assert_prop!(group_props, "group_ts", TimeStamp, (1609459200, 0));

    let channel_props: std::collections::HashMap<_, _> = channel
        .properties()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    assert_prop!(channel_props, "ch_i32", I32, 123456);
    assert_prop!(channel_props, "ch_u64", U64, 9876543210);
    assert_prop!(channel_props, "ch_f64", Double, std::f64::consts::E);
    assert_prop!(channel_props, "ch_bool", Boolean, true);
    assert_prop!(
        channel_props,
        "ch_string",
        String,
        "ChannelLevel".to_string()
    );
    assert_prop!(channel_props, "ch_ts", TimeStamp, (1640995200, 500000000));

    println!("Group properties: {}", group_props.len());
    println!("Channel properties: {}", channel_props.len());

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
