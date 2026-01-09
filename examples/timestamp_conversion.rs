//! TDMS timestamp conversion example
//! 
//! This example demonstrates how to work with TDMS timestamps, including
//! conversion to different formats and understanding the TDMS time epoch.

use std::env;
use std::path::Path;
use tdms_rs::TdmsFile;

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
    
    let file = TdmsFile::load(Path::new(file_path))?;
    
    // Look for timestamp channels
    let mut found_timestamps = false;
    
    for (group_name, group) in &file.groups {
        for (channel_name, channel) in &group.channels {
            if let Some(timestamps) = channel.as_timestamps() {
                found_timestamps = true;
                
                println!("\n📊 Found timestamp channel: {}/{}", group_name, channel_name);
                println!("   Raw timestamp count: {}", timestamps.len());
                
                // Show raw TDMS timestamps
                println!("\n🔢 Raw TDMS Timestamps (first 5):");
                for (i, (seconds, fraction)) in timestamps.iter().take(5).enumerate() {
                    println!("   [{}]: {} seconds + {} fraction", i, seconds, fraction);
                }
                
                // Convert to f64 seconds since 1904
                if let Some(f64_times) = channel.as_timestamps_f64() {
                    println!("\n⏱️  Converted to f64 seconds since 1904 (first 5):");
                    for (i, time) in f64_times.iter().take(5).enumerate() {
                        println!("   [{}]: {:.9} seconds", i, time);
                    }
                    
                    // Show time differences (intervals)
                    if f64_times.len() > 1 {
                        println!("\n📏 Time intervals between samples:");
                        for i in 1..std::cmp::min(6, f64_times.len()) {
                            let interval = f64_times[i] - f64_times[i-1];
                            println!("   Sample {} to {}: {:.9} seconds", i-1, i, interval);
                        }
                    }
                }
                
                // Convert to Unix timestamps
                if let Some(unix_times) = channel.timestamps_to_unix() {
                    println!("\n🌍 Converted to Unix epoch (first 5):");
                    for (i, time) in unix_times.iter().take(5).enumerate() {
                        // Convert to a readable date (simple approximation)
                        let days_since_1970 = (*time / 86400.0) as i64;
                        let year_approx = 1970 + (days_since_1970 / 365);
                        println!("   [{}]: {:.9} (≈ year {})", i, time, year_approx);
                    }
                }
                
                // Show timestamp properties if any
                if !channel.properties.is_empty() {
                    println!("\n📋 Timestamp Channel Properties:");
                    for (prop_name, prop_value) in &channel.properties {
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
        
        // Create a simple example with timestamps
        println!("\n🔧 Creating example timestamp data...");
        create_timestamp_example()?;
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

fn create_timestamp_example() -> Result<(), Box<dyn std::error::Error>> {
    use tdms_rs::writer::TdmsFileWriter;
    use tdms_rs::TdmsData;
    use std::fs;
    
    fs::create_dir_all("examples/output")?;
    
    let mut writer = TdmsFileWriter::new("examples/output/timestamp_example.tdms");
    
    // Add file properties
    writer.add_property("Title", "Timestamp Conversion Example")?;
    writer.add_property("Created", "2026-01-09")?;
    
    let time_group = writer.add_group("Time")?;
    time_group.add_property("Description", "Example timestamp data")?;
    
    // Create some example timestamps
    // These represent times around 2020-01-01 (approximately)
    let base_time = 3660134400i64; // Roughly 2020-01-01 in TDMS epoch
    let timestamps = vec![
        (base_time, 0),                           // Base time
        (base_time + 1, 0),                       // +1 second
        (base_time + 2, 9223372036854775808u64),  // +2.5 seconds
        (base_time + 3, 0),                       // +3 seconds
        (base_time + 4, 4611686018427387904u64),  // +4.25 seconds
    ];
    
    time_group.add_channel("Events", TdmsData::TimeStamp(timestamps))?;
    
    writer.write()?;
    
    println!("✅ Created example file: examples/output/timestamp_example.tdms");
    
    // Now read it back and demonstrate
    let file = TdmsFile::load(Path::new("examples/output/timestamp_example.tdms"))?;
    
    if let Some(channel) = file.get_channel("Time", "Events") {
        println!("\n📊 Example Timestamp Data:");
        
        if let Some(timestamps) = channel.as_timestamps() {
            println!("   Raw timestamps:");
            for (i, (seconds, fraction)) in timestamps.iter().enumerate() {
                println!("     [{}]: {} seconds + {} fraction", i, seconds, fraction);
            }
        }
        
        if let Some(f64_times) = channel.as_timestamps_f64() {
            println!("   As f64 seconds since 1904:");
            for (i, time) in f64_times.iter().enumerate() {
                println!("     [{}]: {:.9}", i, time);
            }
        }
        
        if let Some(unix_times) = channel.timestamps_to_unix() {
            println!("   As Unix timestamps:");
            for (i, time) in unix_times.iter().enumerate() {
                println!("     [{}]: {:.9}", i, time);
            }
        }
    }
    
    Ok(())
}