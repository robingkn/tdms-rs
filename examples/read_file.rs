//! Basic TDMS file reading example
//! 
//! This example demonstrates how to load a TDMS file and print its basic structure.
//! 
//! Usage: cargo run --example read_file -- path/to/file.tdms

use std::env;
use std::path::Path;
use tdms::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get file path from command line arguments
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        // Use a test file if no argument provided
        "tests/fixtures/tdms_corpus/01_minimal/minimal.tdms"
    };

    println!("Loading TDMS file: {}", file_path);
    
    // Load the TDMS file
    let file = TdmsFile::load(Path::new(file_path))?;
    
    println!("Successfully loaded TDMS file!");
    println!("Found {} groups:", file.groups.len());
    
    // Print basic file structure
    for (group_name, group) in &file.groups {
        println!("\n📁 Group: '{}'", group_name);
        println!("   Properties: {}", group.properties.len());
        println!("   Channels: {}", group.channels.len());
        
        // List channels in this group
        for (channel_name, channel) in &group.channels {
            let data_info = match &channel.data {
                Some(data) => format!("{} samples", get_data_len(data)),
                None => "no data".to_string(),
            };
            println!("   📊 Channel '{}': {} properties, {}", 
                     channel_name, channel.properties.len(), data_info);
        }
    }
    
    println!("\n✅ File structure displayed successfully!");
    Ok(())
}

/// Helper function to get the number of data points in TdmsData
fn get_data_len(data: &tdms::TdmsData) -> usize {
    match data {
        tdms::TdmsData::I8(v) => v.len(),
        tdms::TdmsData::I16(v) => v.len(),
        tdms::TdmsData::I32(v) => v.len(),
        tdms::TdmsData::I64(v) => v.len(),
        tdms::TdmsData::U8(v) => v.len(),
        tdms::TdmsData::U16(v) => v.len(),
        tdms::TdmsData::U32(v) => v.len(),
        tdms::TdmsData::U64(v) => v.len(),
        tdms::TdmsData::Float(v) => v.len(),
        tdms::TdmsData::Double(v) => v.len(),
        tdms::TdmsData::String(v) => v.len(),
        tdms::TdmsData::Boolean(v) => v.len(),
        tdms::TdmsData::TimeStamp(v) => v.len(),
    }
}