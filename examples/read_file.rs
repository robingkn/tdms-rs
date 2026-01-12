//! Basic TDMS file reading example
//!
//! This example demonstrates how to load a TDMS file and print its basic structure.
//!
//! Usage: cargo run --example read_file -- path/to/file.tdms

use std::env;
use std::path::Path;
use tdms_rs::TdmsFile;

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
            let data_info = if channel.data_len() > 0 {
                format!(
                    "{} samples ({})",
                    channel.data_len(),
                    channel.data_type_name().unwrap_or("unknown")
                )
            } else {
                "no data".to_string()
            };
            println!(
                "   📊 Channel '{}': {} properties, {}",
                channel_name,
                channel.properties.len(),
                data_info
            );
        }
    }

    println!("\n✅ File structure displayed successfully!");
    Ok(())
}
