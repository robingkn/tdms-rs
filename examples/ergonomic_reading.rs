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
            
            // Current: Type-specific data access
            if let Some(data) = channel.as_f64() {
                println!("   Data (f64): {} samples", data.len());
                if !data.is_empty() {
                    println!("   First value: {:.6}", data[0]);
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
        },
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
            println!("   Data type: {}", channel.data_type_name().unwrap_or("None"));
            println!("   Data length: {}", channel.data_len());
            
            // Show all new helper methods
            if let Some(data) = channel.as_f64() {
                println!("   ✅ as_f64() works: {} values", data.len());
            }
            if let Some(data) = channel.as_f32() {
                println!("   ✅ as_f32() works: {} values", data.len());
            }
            if let Some(data) = channel.as_i8() {
                println!("   ✅ as_i8() works: {} values", data.len());
            }
            if let Some(data) = channel.as_i16() {
                println!("   ✅ as_i16() works: {} values", data.len());
            }
            if let Some(data) = channel.as_i32() {
                println!("   ✅ as_i32() works: {} values", data.len());
            }
            if let Some(data) = channel.as_i64() {
                println!("   ✅ as_i64() works: {} values", data.len());
            }
            if let Some(data) = channel.as_u8() {
                println!("   ✅ as_u8() works: {} values", data.len());
            }
            if let Some(data) = channel.as_u16() {
                println!("   ✅ as_u16() works: {} values", data.len());
            }
            if let Some(data) = channel.as_u32() {
                println!("   ✅ as_u32() works: {} values", data.len());
            }
            if let Some(data) = channel.as_u64() {
                println!("   ✅ as_u64() works: {} values", data.len());
            }
            if let Some(data) = channel.as_bool() {
                println!("   ✅ as_bool() works: {} values", data.len());
            }
            if let Some(data) = channel.as_timestamps() {
                println!("   ✅ as_timestamps() works: {} values", data.len());
            }
            
            // Show numeric conversion
            if let Some(numeric_data) = channel.as_numeric() {
                let avg = numeric_data.iter().sum::<f64>() / numeric_data.len() as f64;
                println!("   Average (as_numeric): {:.6}", avg);
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
        println!("     Group '{}': {} channels", group_name, group.channels.len());
        
        // Show group iteration helper (already exists)
        for (channel_name, channel) in group.iter_channels() {
            println!("       Channel '{}': {} samples", 
                     channel_name, 
                     channel.data_len());
        }
    }
    
    // Show file iteration helper (already exists)
    println!("\n   Using iter_groups():");
    for (group_name, group) in file.iter_groups() {
        println!("     Group '{}' (via iterator)", group_name);
    }
    
    println!("\n✨ Ergonomic reading demonstration complete!");
    println!("💡 All improved patterns shown are now available in v1.0.0!");
    
    Ok(())
}