//! List all channels with detailed metadata
//! 
//! This example shows how to iterate through groups and channels,
//! displaying their properties and data type information.
//! 
//! Usage: cargo run --example list_channels -- path/to/file.tdms

use std::env;
use std::path::Path;
use tdms_rs::{TdmsFile, TdmsData, PropertyValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "tests/fixtures/tdms_corpus/03_datatypes/integers.tdms"
    };

    println!("📋 Listing channels in: {}", file_path);
    
    let file = TdmsFile::load(Path::new(file_path))?;
    
    if file.groups.is_empty() {
        println!("No groups found in the file.");
        return Ok(());
    }
    
    for (group_name, group) in &file.groups {
        println!("\n🗂️  Group: '{}'", group_name);
        
        // Display group properties
        if !group.properties.is_empty() {
            println!("   Group Properties:");
            for (prop_name, prop_value) in &group.properties {
                println!("     • {}: {}", prop_name, format_property_value(prop_value));
            }
        }
        
        // Display channels
        if group.channels.is_empty() {
            println!("   (No channels)");
        } else {
            for (channel_name, channel) in &group.channels {
                println!("\n   📊 Channel: '{}'", channel_name);
                
                // Show data type and count
                match &channel.data {
                    Some(data) => {
                        let (data_type, count) = get_data_info(data);
                        println!("      Data: {} ({} samples)", data_type, count);
                        
                        // Show first few values as preview
                        print_data_preview(data);
                    },
                    None => println!("      Data: None"),
                }
                
                // Show channel properties
                if !channel.properties.is_empty() {
                    println!("      Properties:");
                    for (prop_name, prop_value) in &channel.properties {
                        println!("        • {}: {}", prop_name, format_property_value(prop_value));
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
    }
}

fn get_data_info(data: &TdmsData) -> (&'static str, usize) {
    match data {
        TdmsData::I8(v) => ("i8", v.len()),
        TdmsData::I16(v) => ("i16", v.len()),
        TdmsData::I32(v) => ("i32", v.len()),
        TdmsData::I64(v) => ("i64", v.len()),
        TdmsData::U8(v) => ("u8", v.len()),
        TdmsData::U16(v) => ("u16", v.len()),
        TdmsData::U32(v) => ("u32", v.len()),
        TdmsData::U64(v) => ("u64", v.len()),
        TdmsData::Float(v) => ("f32", v.len()),
        TdmsData::Double(v) => ("f64", v.len()),
        TdmsData::String(v) => ("String", v.len()),
        TdmsData::Boolean(v) => ("bool", v.len()),
        TdmsData::TimeStamp(v) => ("TimeStamp", v.len()),
    }
}

fn print_data_preview(data: &TdmsData) {
    const PREVIEW_COUNT: usize = 3;
    
    match data {
        TdmsData::I8(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::I16(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::I32(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::I64(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::U8(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::U16(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::U32(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::U64(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::Float(v) => {
            if !v.is_empty() {
                let preview: Vec<String> = v.iter().take(PREVIEW_COUNT).map(|x| format!("{:.3}", x)).collect();
                let suffix = if v.len() > PREVIEW_COUNT { ", ..." } else { "" };
                println!("      Preview: [{}{}]", preview.join(", "), suffix);
            }
        },
        TdmsData::Double(v) => {
            if !v.is_empty() {
                let preview: Vec<String> = v.iter().take(PREVIEW_COUNT).map(|x| format!("{:.6}", x)).collect();
                let suffix = if v.len() > PREVIEW_COUNT { ", ..." } else { "" };
                println!("      Preview: [{}{}]", preview.join(", "), suffix);
            }
        },
        TdmsData::String(v) => {
            if !v.is_empty() {
                let preview: Vec<String> = v.iter().take(PREVIEW_COUNT).map(|s| format!("\"{}\"", s)).collect();
                let suffix = if v.len() > PREVIEW_COUNT { ", ..." } else { "" };
                println!("      Preview: [{}{}]", preview.join(", "), suffix);
            }
        },
        TdmsData::Boolean(v) => print_vec_preview(v, PREVIEW_COUNT),
        TdmsData::TimeStamp(v) => {
            if !v.is_empty() {
                let preview: Vec<String> = v.iter().take(PREVIEW_COUNT)
                    .map(|(sec, frac)| format!("({}, {})", sec, frac)).collect();
                let suffix = if v.len() > PREVIEW_COUNT { ", ..." } else { "" };
                println!("      Preview: [{}{}]", preview.join(", "), suffix);
            }
        },
    }
}

fn print_vec_preview<T: std::fmt::Display>(v: &[T], count: usize) {
    if !v.is_empty() {
        let preview: Vec<String> = v.iter().take(count).map(|x| x.to_string()).collect();
        let suffix = if v.len() > count { ", ..." } else { "" };
        println!("      Preview: [{}{}]", preview.join(", "), suffix);
    }
}