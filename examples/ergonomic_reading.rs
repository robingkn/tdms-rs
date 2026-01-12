//! Ergonomic TDMS file reading example
//!
//! This example demonstrates improved ways to access TDMS data using
//! convenience methods and ergonomic patterns. It shows both current
//! and improved approaches for comparison.

use std::env;
use std::path::Path;
use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "tests/fixtures/tdms_corpus/06_properties/all_levels.tdms"
    };

    println!("🔍 Demonstrating ergonomic TDMS reading patterns");
    println!("File: {}", file_path);

    let file = TdmsFile::load(Path::new(file_path))?;

    // === CURRENT API PATTERNS ===
    println!("\n📋 Current API Patterns:");

    // Current: Verbose nested access
    if let Some(group) = file.groups.get("Group") {
        if let Some(channel) = group.channels.get("Channel") {
            println!("✅ Found channel using current API");

            // Current: Type-specific data access using slice-based API
            let expected_count = channel.data_len();
            let mut buffer = vec![0.0f64; expected_count];
            if let Ok(count) = channel.read_f64_into(&mut buffer) {
                println!("   Data (f64): {} samples", count);
                if count > 0 {
                    println!("   First value: {:.6}", buffer[0]);
                }
            }

            // Current: Property access with magic strings
            if let Some(unit) = channel.get_string_property("units") {
                println!("   Unit: {}", unit);
            }

            if let Some(max_val) = channel.get_double_property("max_val") {
                println!("   Max value: {:.3}", max_val);
            }
        }
    }

    // === IMPROVED API PATTERNS (Now Available!) ===
    println!("\n🚀 Improved API Patterns (Now Available!):");

    // Improved: Direct channel access
    if let Some(channel) = file.get_channel("Group", "Channel") {
        println!("✅ Found channel using improved get_channel() method");

        // Show new helper methods
        if let Some(desc) = channel.description() {
            println!("   Description: {}", desc);
        }

        if let Some(sensor) = channel.sensor_type() {
            println!("   Sensor type: {}", sensor);
        }
    }

    // Improved: Error handling with descriptive messages
    match file.try_get_channel("Group", "Channel") {
        Ok(channel) => {
            println!("✅ Channel found with error handling");
            println!("   Data length: {}", channel.data_len());
        }
        Err(e) => println!("❌ Channel access failed: {}", e),
    }

    // Improved: Using property constants
    if let Some(channel) = file.get_channel("Group", "Channel") {
        // Use constants instead of magic strings
        if let Some(unit) = channel.get_string_property(tdms_rs::properties::UNIT_STRING) {
            println!("   Unit (using constant): {}", unit);
        }
    }

    // === DEMONSTRATE EXISTING HELPER METHODS ===
    println!("\n🛠️  Existing Helper Methods:");

    if let Some(group) = file.groups.get("Group") {
        if let Some(channel) = group.channels.get("Channel") {
            // Show data type detection
            println!(
                "   Data type: {}",
                channel.data_type_name().unwrap_or("None")
            );
            println!("   Data length: {}", channel.data_len());

            // Show slice-based reading methods
            let expected_count = channel.data_len();
            match channel.data_type_name() {
                Some("Double") => {
                    let mut buffer = vec![0.0f64; expected_count];
                    if let Ok(count) = channel.read_f64_into(&mut buffer) {
                        println!("   ✅ read_f64_into() works: {} values", count);
                        if count > 0 {
                            let avg = buffer.iter().take(count).sum::<f64>() / count as f64;
                            println!("   Average: {:.6}", avg);
                        }
                    }
                }
                Some("Float") => {
                    let mut buffer = vec![0.0f32; expected_count];
                    if let Ok(count) = channel.read_f32_into(&mut buffer) {
                        println!("   ✅ read_f32_into() works: {} values", count);
                    }
                }
                Some("I8") => {
                    let mut buffer = vec![0i8; expected_count];
                    if let Ok(count) = channel.read_i8_into(&mut buffer) {
                        println!("   ✅ read_i8_into() works: {} values", count);
                    }
                }
                Some("I16") => {
                    let mut buffer = vec![0i16; expected_count];
                    if let Ok(count) = channel.read_i16_into(&mut buffer) {
                        println!("   ✅ read_i16_into() works: {} values", count);
                    }
                }
                Some("I32") => {
                    let mut buffer = vec![0i32; expected_count];
                    if let Ok(count) = channel.read_i32_into(&mut buffer) {
                        println!("   ✅ read_i32_into() works: {} values", count);
                    }
                }
                Some("I64") => {
                    let mut buffer = vec![0i64; expected_count];
                    if let Ok(count) = channel.read_i64_into(&mut buffer) {
                        println!("   ✅ read_i64_into() works: {} values", count);
                    }
                }
                Some("U8") => {
                    let mut buffer = vec![0u8; expected_count];
                    if let Ok(count) = channel.read_u8_into(&mut buffer) {
                        println!("   ✅ read_u8_into() works: {} values", count);
                    }
                }
                Some("U16") => {
                    let mut buffer = vec![0u16; expected_count];
                    if let Ok(count) = channel.read_u16_into(&mut buffer) {
                        println!("   ✅ read_u16_into() works: {} values", count);
                    }
                }
                Some("U32") => {
                    let mut buffer = vec![0u32; expected_count];
                    if let Ok(count) = channel.read_u32_into(&mut buffer) {
                        println!("   ✅ read_u32_into() works: {} values", count);
                    }
                }
                Some("U64") => {
                    let mut buffer = vec![0u64; expected_count];
                    if let Ok(count) = channel.read_u64_into(&mut buffer) {
                        println!("   ✅ read_u64_into() works: {} values", count);
                    }
                }
                Some("Boolean") => {
                    let mut buffer = vec![false; expected_count];
                    if let Ok(count) = channel.read_bool_into(&mut buffer) {
                        println!("   ✅ read_bool_into() works: {} values", count);
                    }
                }
                Some("TimeStamp") => {
                    let mut buffer = vec![(0i64, 0u64); expected_count];
                    if let Ok(count) = channel.read_timestamp_into(&mut buffer) {
                        println!("   ✅ read_timestamp_into() works: {} values", count);
                    }
                }
                _ => {
                    println!("   Data type: {:?}", channel.data_type_name());
                }
            }

            // Show existing and new property helpers
            if let Some(unit) = channel.unit() {
                println!("   ✅ unit() helper works: {}", unit);
            }

            if let Some(increment) = channel.increment() {
                println!("   ✅ increment() helper works: {:.6}", increment);
            }

            if let Some(start_time) = channel.start_time() {
                println!("   ✅ start_time() helper works: {:.6}", start_time);
            }

            if let Some(desc) = channel.description() {
                println!("   ✅ description() helper works: {}", desc);
            }

            if let Some(sensor) = channel.sensor_type() {
                println!("   ✅ sensor_type() helper works: {}", sensor);
            }

            if let Some(count) = channel.sample_count() {
                println!("   ✅ sample_count() helper works: {}", count);
            }
        }
    }

    // === DEMONSTRATE FILE-LEVEL PROPERTIES ===
    println!("\n📄 File-Level Properties:");
    if file.properties.is_empty() {
        println!("   No file-level properties");
    } else {
        for (key, value) in &file.properties {
            println!("   {}: {:?}", key, value);
        }
    }

    // === SHOW ITERATION PATTERNS ===
    println!("\n🔄 Iteration Patterns:");

    // Current: Manual iteration
    println!("   Groups: {}", file.groups.len());
    for (group_name, group) in &file.groups {
        println!(
            "     Group '{}': {} channels",
            group_name,
            group.channels.len()
        );

        // Show group iteration helper (already exists)
        for (channel_name, channel) in group.iter_channels() {
            println!(
                "       Channel '{}': {} samples",
                channel_name,
                channel.data_len()
            );
        }
    }

    // Show file iteration helper (already exists)
    println!("\n   Using iter_groups():");
    for (group_name, _group) in file.iter_groups() {
        println!("     Group '{}' (via iterator)", group_name);
    }

    println!("\n✨ Ergonomic reading demonstration complete!");
    println!("💡 All improved patterns shown are now available in v1.0.0!");

    Ok(())
}
