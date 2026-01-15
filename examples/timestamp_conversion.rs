//! TDMS timestamp conversion example
//!
//! This example demonstrates how to work with TDMS timestamps, including
//! conversion to different formats and understanding the TDMS time epoch.

use std::env;
use std::path::Path;
use tdms_rs::{TdmsDType, TdmsFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        // Use a test file with timestamps if available
        "tests/fixtures/tdms_corpus/07_timestamps/high_precision.tdms"
    };

    println!("⏰ TDMS Timestamp Conversion Example");
    println!("File: {}", file_path);

    let file = TdmsFile::open(Path::new(file_path))?;

    // Look for timestamp channels
    let mut found_timestamps = false;

    for group in file.groups() {
        for channel in group.channels() {
            if channel.dtype() == TdmsDType::TimeStamp {
                found_timestamps = true;

                println!(
                    "\n📊 Found timestamp channel: {}/{}",
                    group.name(),
                    channel.name()
                );

                println!("   len = {}", channel.len());
                println!("   Note: redesigned API does not decode TimeStamp raw data yet.");

                if channel.properties().count() > 0 {
                    println!("\n📋 Timestamp Channel Properties:");
                    for (prop_name, prop_value) in channel.properties() {
                        println!("   {}: {:?}", prop_name, prop_value);
                    }
                }

                break; // Only show first timestamp channel for brevity
            }
        }
        if found_timestamps {
            break;
        }
    }

    if !found_timestamps {
        println!("\n❌ No timestamp channels found in this file.");
        println!("💡 Try running with a file that contains timestamp data:");
        println!("   cargo run --example timestamp_conversion -- path/to/timestamp_file.tdms");
    }

    // Show epoch information
    println!("\n📅 TDMS Timestamp Format Information:");
    println!("   • TDMS Epoch: January 1, 1904, 00:00:00 UTC");
    println!("   • Unix Epoch: January 1, 1970, 00:00:00 UTC");
    println!("   • Offset: 2,082,844,800 seconds (66 years)");
    println!("   • Precision: 2^-64 seconds (sub-nanosecond)");
    println!("   • Format: (seconds_since_1904, fraction_2_64)");

    Ok(())
}
