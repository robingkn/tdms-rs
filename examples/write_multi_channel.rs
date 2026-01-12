//! Multi-channel TDMS file writing example.
//!
//! This example demonstrates creating a TDMS file with multiple channels
//! containing different data types, simulating a typical data acquisition scenario.

use std::fs;
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::TdmsData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating multi-channel TDMS file...");

    // Create a new TDMS file writer
    let mut writer = TdmsFileWriter::new("examples/output/multi_channel.tdms");

    // Add a sensors group
    let sensors = writer.add_group("Sensors")?;

    // Temperature sensor data (double precision)
    sensors.add_channel(
        "Temperature",
        TdmsData::Double(vec![20.1, 21.5, 22.3, 23.0, 22.8, 21.9, 20.5, 19.8]),
    )?;

    // Pressure sensor data (32-bit integers, representing pascals)
    sensors.add_channel(
        "Pressure",
        TdmsData::I32(vec![
            101325, 101330, 101320, 101315, 101310, 101305, 101300, 101295,
        ]),
    )?;

    // Humidity sensor data (single precision floats)
    sensors.add_channel(
        "Humidity",
        TdmsData::Float(vec![45.2, 46.1, 47.0, 48.5, 49.2, 48.8, 47.5, 46.3]),
    )?;

    // Validity flags (booleans)
    sensors.add_channel(
        "Valid",
        TdmsData::Boolean(vec![true, true, true, false, true, true, true, true]),
    )?;

    // Add a digital I/O group
    let digital = writer.add_group("Digital")?;

    // Digital input states (8-bit unsigned integers)
    digital.add_channel(
        "InputStates",
        TdmsData::U8(vec![
            0b00000001, 0b00000011, 0b00000111, 0b00001111, 0b00011111, 0b00111111, 0b01111111,
            0b11111111,
        ]),
    )?;

    // Event labels (strings)
    digital.add_channel(
        "EventLabels",
        TdmsData::String(vec![
            "Start".to_string(),
            "Sensor1_Active".to_string(),
            "Sensor2_Active".to_string(),
            "Warning".to_string(),
            "Normal".to_string(),
            "Calibration".to_string(),
            "Shutdown".to_string(),
            "Stop".to_string(),
        ]),
    )?;

    // Write the file
    writer.write()?;

    println!("✅ Successfully created 'examples/output/multi_channel.tdms'");
    println!("   - 2 groups: 'Sensors', 'Digital'");
    println!("   - 6 channels with different data types");

    // Verify by reading it back
    println!("\n🔍 Verifying by reading the file back...");
    let file = tdms_rs::TdmsFile::load(std::path::Path::new("examples/output/multi_channel.tdms"))?;

    for (group_name, group) in &file.groups {
        println!("   Group: {}", group_name);
        for (channel_name, channel) in &group.channels {
            match channel.ensure_data_loaded()? {
                tdms_rs::TdmsData::Double(values) => {
                    println!(
                        "     Channel '{}': {} double values",
                        channel_name,
                        values.len()
                    );
                    println!(
                        "       Range: {:.2} to {:.2}",
                        values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                        values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
                    );
                }
                tdms_rs::TdmsData::Float(values) => {
                    println!(
                        "     Channel '{}': {} float values",
                        channel_name,
                        values.len()
                    );
                    println!(
                        "       Range: {:.2} to {:.2}",
                        values.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                        values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                    );
                }
                tdms_rs::TdmsData::I32(values) => {
                    println!(
                        "     Channel '{}': {} i32 values",
                        channel_name,
                        values.len()
                    );
                    println!(
                        "       Range: {} to {}",
                        values.iter().min().unwrap(),
                        values.iter().max().unwrap()
                    );
                }
                tdms_rs::TdmsData::U8(values) => {
                    println!(
                        "     Channel '{}': {} u8 values",
                        channel_name,
                        values.len()
                    );
                    println!("       Values: {:?}", values);
                }
                tdms_rs::TdmsData::Boolean(values) => {
                    let true_count = values.iter().filter(|&&x| x).count();
                    println!(
                        "     Channel '{}': {} boolean values",
                        channel_name,
                        values.len()
                    );
                    println!(
                        "       True: {}, False: {}",
                        true_count,
                        values.len() - true_count
                    );
                }
                tdms_rs::TdmsData::String(values) => {
                    println!(
                        "     Channel '{}': {} string values",
                        channel_name,
                        values.len()
                    );
                    println!("       Values: {:?}", values);
                }
                data => println!("     Channel '{}': {:?}", channel_name, data.type_name()),
            }
        }
    }

    println!("\n✨ Round-trip verification successful!");

    Ok(())
}
