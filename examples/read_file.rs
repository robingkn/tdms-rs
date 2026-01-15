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
    let file = TdmsFile::open(Path::new(file_path))?;

    println!("Successfully loaded TDMS file!");
    println!("Found {} groups:", file.groups().count());

    // Print basic file structure
    for group in file.groups() {
        println!("\n📁 Group: '{}'", group.name());
        println!("   Properties: {}", group.properties().count());
        println!("   Channels: {}", group.channels().count());

        for channel in group.channels() {
            println!(
                "   📊 Channel '{}': {} properties, {} samples ({:?})",
                channel.name(),
                channel.properties().count(),
                channel.len(),
                channel.dtype()
            );
        }
    }

    println!("\n✅ File structure displayed successfully!");
    Ok(())
}
