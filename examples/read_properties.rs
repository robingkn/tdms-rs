//! Read and display TDMS properties at all levels
//! 
//! This example demonstrates how to access file, group, and channel properties,
//! showing the hierarchical metadata structure of TDMS files.
//! 
//! Usage: cargo run --example read_properties -- path/to/file.tdms

use std::env;
use std::path::Path;
use tdms::{TdmsFile, PropertyValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "tests/fixtures/tdms_corpus/06_properties/all_levels.tdms"
    };

    println!("🏷️  Reading properties from: {}", file_path);
    
    let file = TdmsFile::load(Path::new(file_path))?;
    
    println!("✅ File loaded successfully!");
    
    // Note: File-level properties are not currently exposed in the public API
    // This is a limitation of the current implementation
    println!("\n📄 File Properties:");
    println!("   (File-level properties not currently accessible via public API)");
    
    // Display group and channel properties
    for (group_name, group) in &file.groups {
        println!("\n📁 Group: '{}'", group_name);
        
        // Group properties
        if group.properties.is_empty() {
            println!("   No group properties");
        } else {
            println!("   Group Properties ({}):", group.properties.len());
            for (prop_name, prop_value) in &group.properties {
                println!("     • {}: {}", prop_name, format_property_detailed(prop_value));
            }
        }
        
        // Channel properties
        for (channel_name, channel) in &group.channels {
            println!("\n   📊 Channel: '{}'", channel_name);
            
            if channel.properties.is_empty() {
                println!("      No channel properties");
            } else {
                println!("      Channel Properties ({}):", channel.properties.len());
                for (prop_name, prop_value) in &channel.properties {
                    println!("        • {}: {}", prop_name, format_property_detailed(prop_value));
                }
            }
            
            // Show common TDMS properties if present
            show_common_properties(&channel.properties);
        }
    }
    
    println!("\n✅ Property display complete!");
    Ok(())
}

fn format_property_detailed(value: &PropertyValue) -> String {
    match value {
        PropertyValue::I8(v) => format!("{} (i8)", v),
        PropertyValue::I16(v) => format!("{} (i16)", v),
        PropertyValue::I32(v) => format!("{} (i32)", v),
        PropertyValue::I64(v) => format!("{} (i64)", v),
        PropertyValue::U8(v) => format!("{} (u8)", v),
        PropertyValue::U16(v) => format!("{} (u16)", v),
        PropertyValue::U32(v) => format!("{} (u32)", v),
        PropertyValue::U64(v) => format!("{} (u64)", v),
        PropertyValue::Float(v) => {
            if v.is_nan() {
                "NaN (f32)".to_string()
            } else if v.is_infinite() {
                if *v > 0.0 {
                    "Infinity (f32)".to_string()
                } else {
                    "-Infinity (f32)".to_string()
                }
            } else {
                format!("{:.6} (f32)", v)
            }
        },
        PropertyValue::Double(v) => {
            if v.is_nan() {
                "NaN (f64)".to_string()
            } else if v.is_infinite() {
                if *v > 0.0 {
                    "Infinity (f64)".to_string()
                } else {
                    "-Infinity (f64)".to_string()
                }
            } else {
                format!("{:.12} (f64)", v)
            }
        },
        PropertyValue::String(v) => {
            if v.len() > 50 {
                format!("\"{}...\" (String, {} chars)", &v[..47], v.len())
            } else {
                format!("\"{}\" (String)", v)
            }
        },
        PropertyValue::Boolean(v) => format!("{} (bool)", v),
    }
}

fn show_common_properties(properties: &std::collections::HashMap<String, PropertyValue>) {
    // Check for common TDMS channel properties and explain their meaning
    let common_props = [
        ("wf_increment", "Waveform time increment (sampling interval)"),
        ("wf_start_offset", "Waveform start time offset"),
        ("wf_samples", "Number of samples in waveform"),
        ("wf_start_time", "Waveform start time"),
        ("wf_unit_string", "Physical unit of measurement"),
        ("NI_ArrayColumn", "Array column information"),
        ("NI_ChannelLength", "Channel data length"),
        ("datatype", "TDMS data type identifier"),
    ];
    
    let mut found_common = false;
    for (prop_name, description) in &common_props {
        if let Some(value) = properties.get(*prop_name) {
            if !found_common {
                println!("      📋 Common TDMS Properties:");
                found_common = true;
            }
            println!("        ℹ️  {}: {} - {}", prop_name, format_property_detailed(value), description);
        }
    }
    
    // Look for custom properties (not in the common list)
    let custom_props: Vec<_> = properties.keys()
        .filter(|key| !common_props.iter().any(|(common_key, _)| common_key == key))
        .collect();
    
    if !custom_props.is_empty() {
        println!("      🔧 Custom Properties: {}", custom_props.len());
        for prop_name in custom_props {
            if let Some(value) = properties.get(prop_name) {
                println!("        • {}: {}", prop_name, format_property_detailed(value));
            }
        }
    }
}