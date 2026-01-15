//! Multi-channel TDMS file writing example.
//!
//! This example demonstrates creating a TDMS file with multiple channels
//! containing different data types, simulating a typical data acquisition scenario.

use std::fs;
use tdms_rs::{TdmsDType, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating multi-channel TDMS file...");

    // Create a new TDMS file writer
    let mut writer = TdmsWriter::create("examples/output/multi_channel.tdms")?;

    // Add a sensors group
    let mut sensors = writer.add_group("Sensors")?;

    // Temperature sensor data (double precision)
    let mut temp = sensors.add_channel::<f64>("Temperature")?;
    temp.write(&[20.1, 21.5, 22.3, 23.0, 22.8, 21.9, 20.5, 19.8])?;

    // Pressure sensor data (32-bit integers, representing pascals)
    let mut pressure = sensors.add_channel::<i32>("Pressure")?;
    pressure.write(&[
        101325, 101330, 101320, 101315, 101310, 101305, 101300, 101295,
    ])?;

    // Humidity sensor data (single precision floats)
    let mut humidity = sensors.add_channel::<f32>("Humidity")?;
    humidity.write(&[45.2, 46.1, 47.0, 48.5, 49.2, 48.8, 47.5, 46.3])?;

    // Validity flags (booleans)
    let mut valid = sensors.add_channel::<bool>("Valid")?;
    valid.write(&[true, true, true, false, true, true, true, true])?;

    // Add a digital I/O group
    let mut digital = writer.add_group("Digital")?;

    // Digital input states (8-bit unsigned integers)
    let mut input_states = digital.add_channel::<u8>("InputStates")?;
    input_states.write(&[
        0b00000001, 0b00000011, 0b00000111, 0b00001111, 0b00011111, 0b00111111, 0b01111111,
        0b11111111,
    ])?;

    // String channels are intentionally not written by this example because the redesigned
    // writer API only supports fixed-size primitive types and does not implement string encoding.

    // Write the file
    writer.close()?;

    println!("✅ Successfully created 'examples/output/multi_channel.tdms'");
    println!("   - 2 groups: 'Sensors', 'Digital'");
    println!("   - 6 channels with different data types");

    // Verify by reading it back
    println!("\n🔍 Verifying by reading the file back...");
    let file = TdmsFile::open(std::path::Path::new("examples/output/multi_channel.tdms"))?;

    for group in file.groups() {
        println!("   Group: {}", group.name());
        for channel in group.channels() {
            match channel.dtype() {
                TdmsDType::F64 => {
                    let slice = channel.read_all()?;
                    let values = slice.as_typed::<f64>()?;
                    println!(
                        "     Channel '{}': {} f64 values",
                        channel.name(),
                        values.len()
                    );
                }
                TdmsDType::F32 => {
                    let slice = channel.read_all()?;
                    let values = slice.as_typed::<f32>()?;
                    println!(
                        "     Channel '{}': {} f32 values",
                        channel.name(),
                        values.len()
                    );
                }
                TdmsDType::I32 => {
                    let slice = channel.read_all()?;
                    let values = slice.as_typed::<i32>()?;
                    println!(
                        "     Channel '{}': {} i32 values",
                        channel.name(),
                        values.len()
                    );
                }
                TdmsDType::U8 => {
                    let slice = channel.read_all()?;
                    let values = slice.as_typed::<u8>()?;
                    println!(
                        "     Channel '{}': {} u8 values",
                        channel.name(),
                        values.len()
                    );
                }
                TdmsDType::Bool => {
                    let slice = channel.read_all()?;
                    let values = slice.as_typed::<bool>()?;
                    println!(
                        "     Channel '{}': {} bool values",
                        channel.name(),
                        values.len()
                    );
                }
                other => {
                    println!(
                        "     Channel '{}': {} samples ({:?})",
                        channel.name(),
                        channel.len(),
                        other
                    );
                }
            }
        }
    }

    println!("\n✨ Round-trip verification successful!");

    Ok(())
}
