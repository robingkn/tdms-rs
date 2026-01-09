//! Comprehensive TDMS file writing example with all supported data types.
//! 
//! This example demonstrates creating a TDMS file with every supported data type,
//! including edge cases and special values. This is useful for testing compatibility
//! and understanding the full range of TDMS capabilities.

use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::{TdmsData, PropertyValue};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;
    
    println!("📝 Creating comprehensive TDMS file with all data types...");
    
    // Create a new TDMS file writer
    let mut writer = TdmsFileWriter::new("examples/output/all_types.tdms");
    
    // Add file-level properties (using new From<T> conversions)
    writer.add_property("Title", "Comprehensive Data Type Example")?;
    writer.add_property("Test_Version", 1i32)?;
    
    // === SIGNED INTEGER TYPES ===
    let integers = writer.add_group("Signed_Integers");
    integers.add_property("Description", "All signed integer types with edge cases")?;
    
    // 8-bit signed integers
    integers.add_channel("Int8", TdmsData::I8(vec![
        i8::MIN, -100, -1, 0, 1, 100, i8::MAX
    ]));
    
    // 16-bit signed integers
    integers.add_channel("Int16", TdmsData::I16(vec![
        i16::MIN, -1000, -1, 0, 1, 1000, i16::MAX
    ]));
    
    // 32-bit signed integers
    integers.add_channel("Int32", TdmsData::I32(vec![
        i32::MIN, -1000000, -1, 0, 1, 1000000, i32::MAX
    ]));
    
    // 64-bit signed integers
    integers.add_channel("Int64", TdmsData::I64(vec![
        i64::MIN, -1000000000000, -1, 0, 1, 1000000000000, i64::MAX
    ]));
    
    // === UNSIGNED INTEGER TYPES ===
    let unsigned = writer.add_group("Unsigned_Integers");
    unsigned.add_property("Description", "All unsigned integer types with full range")?;
    
    // 8-bit unsigned integers
    unsigned.add_channel("UInt8", TdmsData::U8(vec![
        0, 1, 127, 128, 200, 254, u8::MAX
    ]));
    
    // 16-bit unsigned integers
    unsigned.add_channel("UInt16", TdmsData::U16(vec![
        0, 1, 1000, 32767, 32768, 60000, u16::MAX
    ]));
    
    // 32-bit unsigned integers
    unsigned.add_channel("UInt32", TdmsData::U32(vec![
        0, 1, 1000000, 2147483647, 2147483648, 4000000000, u32::MAX
    ]));
    
    // 64-bit unsigned integers
    unsigned.add_channel("UInt64", TdmsData::U64(vec![
        0, 1, 1000000000000, u64::MAX / 2, u64::MAX - 1, u64::MAX
    ]));
    
    // === FLOATING POINT TYPES ===
    let floats = writer.add_group("Floating_Point");
    floats.add_property("Description", "Floating point types with special values")?;
    
    // 32-bit floating point with special values
    floats.add_channel("Float32", TdmsData::Float(vec![
        f32::NEG_INFINITY,
        -1000.0,
        -1.0,
        -0.0,
        0.0,
        1.0,
        1000.0,
        f32::INFINITY,
        f32::NAN,
    ]));
    
    // 64-bit floating point with special values
    floats.add_channel("Float64", TdmsData::Double(vec![
        f64::NEG_INFINITY,
        -1000.0,
        -1.0,
        -0.0,
        0.0,
        1.0,
        1000.0,
        f64::INFINITY,
        f64::NAN,
    ]));
    
    // High precision values
    floats.add_channel("High_Precision", TdmsData::Double(vec![
        std::f64::consts::PI,
        std::f64::consts::E,
        std::f64::consts::SQRT_2,
        1.23456789012345e-100,
        1.23456789012345e100,
    ]));
    
    // === BOOLEAN TYPE ===
    let booleans = writer.add_group("Booleans");
    booleans.add_property("Description", "Boolean values and patterns")?;
    
    booleans.add_channel("Simple", TdmsData::Boolean(vec![
        true, false, true, false, true
    ]));
    
    booleans.add_channel("All_True", TdmsData::Boolean(vec![
        true, true, true, true, true
    ]));
    
    booleans.add_channel("All_False", TdmsData::Boolean(vec![
        false, false, false, false, false
    ]));
    
    booleans.add_channel("Pattern", TdmsData::Boolean(vec![
        true, true, false, false, true, true, false, false
    ]));
    
    // === STRING TYPE ===
    let strings = writer.add_group("Strings");
    strings.add_property("Description", "String data with various lengths and content")?;
    
    strings.add_channel("Basic", TdmsData::String(vec![
        "Hello".to_string(),
        "World".to_string(),
        "TDMS".to_string(),
        "File".to_string(),
        "Format".to_string(),
    ]));
    
    strings.add_channel("Mixed_Length", TdmsData::String(vec![
        "A".to_string(),
        "Short".to_string(),
        "Medium length string".to_string(),
        "This is a much longer string with more content to test variable length handling".to_string(),
        "".to_string(), // Empty string
    ]));
    
    strings.add_channel("Special_Characters", TdmsData::String(vec![
        "Numbers: 123456789".to_string(),
        "Symbols: !@#$%^&*()".to_string(),
        "Unicode: αβγδε".to_string(),
        "Spaces and\ttabs\nand newlines".to_string(),
        "Quotes: \"single\" and 'double'".to_string(),
    ]));
    
    // === TIMESTAMP TYPE ===
    let timestamps = writer.add_group("Timestamps");
    timestamps.add_property("Description", "TDMS timestamp format (seconds since 1904-01-01)")?;
    timestamps.add_property("Epoch", "1904-01-01 00:00:00 UTC")?;
    
    // TDMS timestamps: (seconds_since_1904, fraction_2_64)
    timestamps.add_channel("Basic_Times", TdmsData::TimeStamp(vec![
        (0, 0),                    // 1904-01-01 00:00:00.000
        (1000, 0),                 // 1904-01-01 00:16:40.000
        (86400, 0),                // 1904-01-02 00:00:00.000
        (3155760000, 0),           // ~2004-01-01 00:00:00.000
        (3155760000, 9223372036854775808u64), // ~2004-01-01 00:00:00.500 (half second)
    ]));
    
    timestamps.add_channel("High_Precision", TdmsData::TimeStamp(vec![
        (1000, 0),                           // Base time
        (1000, 1844674407370955161u64),      // +0.1 seconds
        (1000, 3689348814741910323u64),      // +0.2 seconds
        (1000, 18446744073709551615u64),     // +0.999999999999999999 seconds
    ]));
    
    // === MIXED DATA GROUP ===
    let mixed = writer.add_group("Mixed_Data");
    mixed.add_property("Description", "Different data types in one group")?;
    mixed.add_property("Channel_Count", 6i32)?;
    
    // Add one channel of each major type
    mixed.add_channel("Integers", TdmsData::I32(vec![1, 2, 3, 4, 5]));
    mixed.add_channel("Floats", TdmsData::Double(vec![1.1, 2.2, 3.3, 4.4, 5.5]));
    mixed.add_channel("Flags", TdmsData::Boolean(vec![true, false, true, false, true]));
    mixed.add_channel("Labels", TdmsData::String(vec![
        "First".to_string(), "Second".to_string(), "Third".to_string(), "Fourth".to_string(), "Fifth".to_string()
    ]));
    mixed.add_channel("Bytes", TdmsData::U8(vec![0x01, 0x02, 0x04, 0x08, 0x10]));
    mixed.add_channel("Events", TdmsData::TimeStamp(vec![
        (1000, 0), (1001, 0), (1002, 0), (1003, 0), (1004, 0)
    ]));
    
    // Write the file
    writer.write()?;
    
    println!("✅ Successfully created 'examples/output/all_types.tdms'");
    println!("   - 6 groups with comprehensive data type coverage");
    println!("   - All TDMS data types represented");
    println!("   - Edge cases and special values included");
    
    // Verify by reading it back
    println!("\n🔍 Verifying by reading the file back...");
    let file = tdms_rs::TdmsFile::load(std::path::Path::new("examples/output/all_types.tdms"))?;
    
    println!("   File contains {} groups:", file.groups.len());
    
    for (group_name, group) in &file.groups {
        println!("     Group '{}': {} channels", group_name, group.channels.len());
        
        // Show group properties if any
        if !group.properties.is_empty() {
            for (prop_name, prop_value) in &group.properties {
                match prop_value {
                    tdms_rs::PropertyValue::String(s) => println!("       Property {}: \"{}\"", prop_name, s),
                    _ => println!("       Property {}: {:?}", prop_name, prop_value),
                }
            }
        }
        
        // Show channel summary
        for (channel_name, channel) in &group.channels {
            if let Some(data) = &channel.data {
                let type_name = match data {
                    tdms_rs::TdmsData::I8(_) => "I8",
                    tdms_rs::TdmsData::I16(_) => "I16", 
                    tdms_rs::TdmsData::I32(_) => "I32",
                    tdms_rs::TdmsData::I64(_) => "I64",
                    tdms_rs::TdmsData::U8(_) => "U8",
                    tdms_rs::TdmsData::U16(_) => "U16",
                    tdms_rs::TdmsData::U32(_) => "U32",
                    tdms_rs::TdmsData::U64(_) => "U64",
                    tdms_rs::TdmsData::Float(_) => "Float",
                    tdms_rs::TdmsData::Double(_) => "Double",
                    tdms_rs::TdmsData::Boolean(_) => "Boolean",
                    tdms_rs::TdmsData::String(_) => "String",
                    tdms_rs::TdmsData::TimeStamp(_) => "TimeStamp",
                };
                
                let count = match data {
                    tdms_rs::TdmsData::I8(v) => v.len(),
                    tdms_rs::TdmsData::I16(v) => v.len(),
                    tdms_rs::TdmsData::I32(v) => v.len(),
                    tdms_rs::TdmsData::I64(v) => v.len(),
                    tdms_rs::TdmsData::U8(v) => v.len(),
                    tdms_rs::TdmsData::U16(v) => v.len(),
                    tdms_rs::TdmsData::U32(v) => v.len(),
                    tdms_rs::TdmsData::U64(v) => v.len(),
                    tdms_rs::TdmsData::Float(v) => v.len(),
                    tdms_rs::TdmsData::Double(v) => v.len(),
                    tdms_rs::TdmsData::Boolean(v) => v.len(),
                    tdms_rs::TdmsData::String(v) => v.len(),
                    tdms_rs::TdmsData::TimeStamp(v) => v.len(),
                };
                
                println!("       Channel '{}': {} {} values", channel_name, count, type_name);
            }
        }
    }
    
    println!("\n✨ Comprehensive data type verification successful!");
    println!("🎯 This file demonstrates the full capabilities of the TDMS writer API.");
    
    Ok(())
}