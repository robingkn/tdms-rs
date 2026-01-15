//! List all channels with detailed metadata
//!
//! This example shows how to iterate through groups and channels,
//! displaying their properties and data type information.
//!
//! Usage: cargo run --example list_channels -- path/to/file.tdms

use std::env;
use std::path::Path;
use tdms_rs::{PropertyValue, TdmsFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "tests/fixtures/tdms_corpus/03_datatypes/integers.tdms"
    };

    println!("📋 Listing channels in: {}", file_path);

    let file = TdmsFile::open(Path::new(file_path))?;

    if file.groups().count() == 0 {
        println!("No groups found in the file.");
        return Ok(());
    }

    for group in file.groups() {
        println!("\n🗂️  Group: '{}'", group.name());

        // Display group properties
        if group.properties().count() > 0 {
            println!("   Group Properties:");
            for (prop_name, prop_value) in group.properties() {
                println!(
                    "     • {}: {}",
                    prop_name,
                    format_property_value(prop_value)
                );
            }
        }

        // Display channels
        if group.channels().count() == 0 {
            println!("   (No channels)");
        } else {
            for channel in group.channels() {
                println!("\n   📊 Channel: '{}'", channel.name());
                println!("      Data: {:?} ({} samples)", channel.dtype(), channel.len());

                if channel.properties().count() > 0 {
                    println!("      Properties:");
                    for (prop_name, prop_value) in channel.properties() {
                        println!(
                            "        • {}: {}",
                            prop_name,
                            format_property_value(prop_value)
                        );
                    }
                }
            }
        }
    }

    println!("\n✅ Channel listing complete!");
    Ok(())
}

fn format_property_value(value: &PropertyValue) -> String {
    match value {
        PropertyValue::I8(v) => v.to_string(),
        PropertyValue::I16(v) => v.to_string(),
        PropertyValue::I32(v) => v.to_string(),
        PropertyValue::I64(v) => v.to_string(),
        PropertyValue::U8(v) => v.to_string(),
        PropertyValue::U16(v) => v.to_string(),
        PropertyValue::U32(v) => v.to_string(),
        PropertyValue::U64(v) => v.to_string(),
        PropertyValue::Float(v) => format!("{:.3}", v),
        PropertyValue::Double(v) => format!("{:.6}", v),
        PropertyValue::String(v) => format!("\"{}\"", v),
        PropertyValue::Boolean(v) => v.to_string(),
        PropertyValue::TimeStamp((seconds, fraction)) => {
            format!("timestamp({}, {})", seconds, fraction)
        }
    }
}
