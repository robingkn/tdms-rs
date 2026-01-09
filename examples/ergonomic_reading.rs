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
    
    // === IMPROVED API PATTERNS (Future) ===
    println!("\n🚀 Improved API Patterns (Proposed):");
    
    // TODO: These methods don't exist yet - this shows the proposed API
    /*
    // Improved: Direct channel access
    if let Some(channel) = file.get_channel("Group", "Channel") {
        println!("✅ Found channel using improved API");
        
        // Improved: Generic property access (proposed)
        let unit: Option<String> = channel.get_property("units");
        let max_val: Option<f64> = channel.get_property("max_val");
        
        println!("   Unit: {}", unit.unwrap_or_else(|| "unknown".to_string()));
        println!("   Max value: {:.3}", max_val.unwrap_or(0.0));
    }
    
    // Improved: Error handling with descriptive messages
    match file.try_get_channel("Group", "Channel") {
        Ok(channel) => println!("✅ Channel found with error handling"),
        Err(e) => println!("❌ Channel access failed: {}", e),
    }
    
    // Improved: Well-known properties with constants
    if let Some(channel) = file.get_channel("Group", "Channel") {
        if let Some(unit) = channel.unit() {
            println!("   Unit (well-known property): {}", unit);
        }
        
        if let Some(description) = channel.description() {
            println!("   Description: {}", description);
        }
    }
    */
    
    // === DEMONSTRATE EXISTING HELPER METHODS ===
    println!("\n🛠️  Existing Helper Methods:");
    
    if let Some(group) = file.groups.get("Group") {
        if let Some(channel) = group.channels.get("Channel") {
            // Show data type detection
            println!("   Data type: {}", channel.data_type_name().unwrap_or("None"));
            println!("   Data length: {}", channel.data_len());
            
            // Show existing helper methods
            if let Some(data) = channel.as_f64() {
                println!("   ✅ as_f64() works: {} values", data.len());
            }
            
            if channel.as_f32().is_some() {
                println!("   ✅ as_f32() available");
            }
            
            if channel.as_i32().is_some() {
                println!("   ✅ as_i32() available");
            }
            
            if channel.as_string().is_some() {
                println!("   ✅ as_string() available");
            }
            
            // Show numeric conversion
            if let Some(numeric_data) = channel.as_numeric() {
                let avg = numeric_data.iter().sum::<f64>() / numeric_data.len() as f64;
                println!("   Average (as_numeric): {:.6}", avg);
            }
            
            // Show existing property helpers
            if let Some(unit) = channel.unit() {
                println!("   ✅ unit() helper works: {}", unit);
            }
            
            if let Some(increment) = channel.increment() {
                println!("   ✅ increment() helper works: {:.6}", increment);
            }
            
            if let Some(start_time) = channel.start_time() {
                println!("   ✅ start_time() helper works: {:.6}", start_time);
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
    println!("💡 Note: Some improved patterns shown are proposed for future versions.");
    
    Ok(())
}