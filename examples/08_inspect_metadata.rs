//! Demonstrates how to inspect TDMS file metadata without loading data.
//! Shows listing groups, channels, properties, and data types.

use std::path::Path;
use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("inspect_metadata.tdms");

    // Create a file with varied metadata for inspection
    {
        let mut writer = TdmsWriter::create(path)?;
        writer.add_property("file_name", PropertyValue::String("Demo File".into()))?;
        writer.add_property("version", PropertyValue::I32(1))?;

        let mut group1 = writer.add_group("Sensors")?;
        group1.add_property("location", PropertyValue::String("Lab A".into()))?;
        group1.add_property("active", PropertyValue::Boolean(true))?;

        let mut ch1 = group1.add_channel::<f64>("Temperature")?;
        ch1.add_property("unit", PropertyValue::String("Celsius".into()))?;
        ch1.add_property("range", PropertyValue::Double(100.0))?;
        ch1.write(&[20.0, 21.0, 22.0])?;

        let mut ch2 = group1.add_channel::<i32>("Pressure")?;
        ch2.add_property("unit", PropertyValue::String("kPa".into()))?;
        ch2.write(&[101, 102, 103])?;

        let mut group2 = writer.add_group("Control")?;
        group2.add_property("mode", PropertyValue::String("Auto".into()))?;

        let mut ch3 = group2.add_channel::<bool>("Status")?;
        ch3.add_property("description", PropertyValue::String("System OK".into()))?;
        ch3.write(&[true, false, true])?;

        writer.close()?;
    }

    // Inspect the file metadata
    let file = TdmsFile::open(path)?;

    println!("=== File Properties ===");
    for (name, value) in file.properties() {
        println!("  {}: {}", name, value);
    }

    println!("\n=== Groups ===");
    for group in file.groups() {
        println!("Group: '{}'", group.name());

        println!("  Properties:");
        for (name, value) in group.properties() {
            println!("    {}: {}", name, value);
        }

        println!("  Channels:");
        for channel in group.channels() {
            println!(
                "    '{}' (type: {:?}, len: {})",
                channel.name(),
                channel.dtype(),
                channel.len()
            );

            if channel.properties().count() > 0 {
                println!("      Properties:");
                for (name, value) in channel.properties() {
                    println!("        {}: {}", name, value);
                }
            }
        }
        println!();
    }

    // Demonstrate accessing specific items
    if let Some(group) = file.group("Sensors") {
        if let Some(channel) = group.channel("Temperature") {
            println!(
                "Direct access: {}.{} has {} samples of type {:?}",
                group.name(),
                channel.name(),
                channel.len(),
                channel.dtype()
            );
        }
    }

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
