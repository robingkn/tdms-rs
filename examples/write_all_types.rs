//! Comprehensive TDMS file writing example with all supported data types.
//!
//! This example demonstrates creating a TDMS file with every supported numeric data type,
//! including edge cases and special values. This is useful for testing compatibility
//! and understanding the full range of TDMS capabilities.

use std::fs;
use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating comprehensive TDMS file with all data types...");

    // Create a new TDMS file writer
    let mut writer = TdmsWriter::create("examples/output/all_types.tdms")?;

    // Add file-level properties
    writer.add_property(
        "Title",
        PropertyValue::String("Comprehensive Data Type Example".into()),
    )?;
    writer.add_property("Test_Version", PropertyValue::I32(1))?;

    // === SIGNED INTEGER TYPES ===
    {
        let mut integers = writer.add_group("Signed_Integers")?;
        integers.add_property(
            "Description",
            PropertyValue::String("All signed integer types with edge cases".into()),
        )?;

        // 8-bit signed integers
        let mut ch_i8 = integers.add_channel::<i8>("Int8")?;
        ch_i8.write(&[i8::MIN, -100, -1, 0, 1, 100, i8::MAX])?;

        // 16-bit signed integers
        let mut ch_i16 = integers.add_channel::<i16>("Int16")?;
        ch_i16.write(&[i16::MIN, -1000, -1, 0, 1, 1000, i16::MAX])?;

        // 32-bit signed integers
        let mut ch_i32 = integers.add_channel::<i32>("Int32")?;
        ch_i32.write(&[i32::MIN, -1000000, -1, 0, 1, 1000000, i32::MAX])?;

        // 64-bit signed integers
        let mut ch_i64 = integers.add_channel::<i64>("Int64")?;
        ch_i64.write(&[i64::MIN, -1000000000000, -1, 0, 1, 1000000000000, i64::MAX])?;
    }

    // === UNSIGNED INTEGER TYPES ===
    {
        let mut unsigned = writer.add_group("Unsigned_Integers")?;
        unsigned.add_property(
            "Description",
            PropertyValue::String("All unsigned integer types with full range".into()),
        )?;

        // 8-bit unsigned integers
        let mut ch_u8 = unsigned.add_channel::<u8>("UInt8")?;
        ch_u8.write(&[0, 1, 127, 128, 200, 254, u8::MAX])?;

        // 16-bit unsigned integers
        let mut ch_u16 = unsigned.add_channel::<u16>("UInt16")?;
        ch_u16.write(&[0, 1, 1000, 32767, 32768, 60000, u16::MAX])?;

        // 32-bit unsigned integers
        let mut ch_u32 = unsigned.add_channel::<u32>("UInt32")?;
        ch_u32.write(&[0, 1, 1000000, 2147483647, 2147483648, 4000000000, u32::MAX])?;

        // 64-bit unsigned integers
        let mut ch_u64 = unsigned.add_channel::<u64>("UInt64")?;
        ch_u64.write(&[0, 1, 1000000000000, u64::MAX / 2, u64::MAX - 1, u64::MAX])?;
    }

    // === FLOATING POINT TYPES ===
    {
        let mut floats = writer.add_group("Floating_Point")?;
        floats.add_property(
            "Description",
            PropertyValue::String("Floating point types with special values".into()),
        )?;

        // 32-bit floating point with special values
        let mut ch_f32 = floats.add_channel::<f32>("Float32")?;
        ch_f32.write(&[
            f32::NEG_INFINITY,
            -1000.0,
            -1.0,
            -0.0,
            0.0,
            1.0,
            1000.0,
            f32::INFINITY,
            f32::NAN,
        ])?;

        // 64-bit floating point with special values
        let mut ch_f64 = floats.add_channel::<f64>("Float64")?;
        ch_f64.write(&[
            f64::NEG_INFINITY,
            -1000.0,
            -1.0,
            -0.0,
            0.0,
            1.0,
            1000.0,
            f64::INFINITY,
            f64::NAN,
        ])?;

        // High precision values
        let mut ch_hp = floats.add_channel::<f64>("High_Precision")?;
        ch_hp.write(&[
            std::f64::consts::PI,
            std::f64::consts::E,
            std::f64::consts::SQRT_2,
            1.23456789012345e-100,
            1.23456789012345e100,
        ])?;
    }

    // === BOOLEAN TYPE ===
    {
        let mut booleans = writer.add_group("Booleans")?;
        booleans.add_property(
            "Description",
            PropertyValue::String("Boolean values and patterns".into()),
        )?;

        let mut ch_simple = booleans.add_channel::<bool>("Simple")?;
        ch_simple.write(&[true, false, true, false, true])?;

        let mut ch_true = booleans.add_channel::<bool>("All_True")?;
        ch_true.write(&[true, true, true, true, true])?;

        let mut ch_false = booleans.add_channel::<bool>("All_False")?;
        ch_false.write(&[false, false, false, false, false])?;

        let mut ch_pattern = booleans.add_channel::<bool>("Pattern")?;
        ch_pattern.write(&[true, true, false, false, true, true, false, false])?;
    }

    // === MIXED DATA GROUP ===
    {
        let mut mixed = writer.add_group("Mixed_Data")?;
        mixed.add_property(
            "Description",
            PropertyValue::String("Different data types in one group".into()),
        )?;
        mixed.add_property("Channel_Count", PropertyValue::I32(4))?;

        // Add one channel of each major numeric type
        let mut ch_int = mixed.add_channel::<i32>("Integers")?;
        ch_int.write(&[1, 2, 3, 4, 5])?;

        let mut ch_flt = mixed.add_channel::<f64>("Floats")?;
        ch_flt.write(&[1.1, 2.2, 3.3, 4.4, 5.5])?;

        let mut ch_flags = mixed.add_channel::<bool>("Flags")?;
        ch_flags.write(&[true, false, true, false, true])?;

        let mut ch_bytes = mixed.add_channel::<u8>("Bytes")?;
        ch_bytes.write(&[0x01, 0x02, 0x04, 0x08, 0x10])?;
    }

    // Write the file
    writer.close()?;

    println!("✅ Successfully created 'examples/output/all_types.tdms'");
    println!("   - 5 groups with comprehensive data type coverage");
    println!("   - All numeric TDMS data types represented");
    println!("   - Edge cases and special values included");

    // Verify by reading it back
    println!("\n🔍 Verifying by reading the file back...");
    let file = TdmsFile::open(std::path::Path::new("examples/output/all_types.tdms"))?;

    println!("   File contains {} groups:", file.groups().count());

    for group in file.groups() {
        println!(
            "     Group '{}': {} channels",
            group.name(),
            group.channels().count()
        );

        // Show group properties if any
        for (prop_name, prop_value) in group.properties() {
            match prop_value {
                PropertyValue::String(s) => {
                    println!("       Property {}: \"{}\"", prop_name, s)
                }
                _ => println!("       Property {}: {:?}", prop_name, prop_value),
            }
        }

        // Show channel summary
        for channel in group.channels() {
            println!(
                "       Channel '{}': {} {:?} values",
                channel.name(),
                channel.len(),
                channel.dtype()
            );
        }
    }

    println!("\n✨ Comprehensive data type verification successful!");
    println!("🎯 This file demonstrates the full capabilities of the TDMS writer API.");

    Ok(())
}
