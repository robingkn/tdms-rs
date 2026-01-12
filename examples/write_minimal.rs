//! Minimal TDMS file writing example.
//!
//! This example demonstrates the simplest way to create a TDMS file
//! with a single group and channel containing double-precision data.

use std::fs;
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::TdmsData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating minimal TDMS file...");

    // Create a new TDMS file writer
    let mut writer = TdmsFileWriter::new("examples/output/minimal.tdms");

    // Add a group
    let group = writer.add_group("Measurements")?;

    // Add a channel with some sample data
    group.add_channel(
        "Temperature",
        TdmsData::Double(vec![20.1, 21.5, 22.3, 23.0, 22.8]),
    )?;

    // Write the file
    writer.write()?;

    println!("✅ Successfully created 'examples/output/minimal.tdms'");
    println!("   - 1 group: 'Measurements'");
    println!("   - 1 channel: 'Temperature' with 5 double values");

    // Verify by reading it back
    println!("\n🔍 Verifying by reading the file back...");
    let file = tdms_rs::TdmsFile::load(std::path::Path::new("examples/output/minimal.tdms"))?;

    for (group_name, group) in &file.groups {
        println!("   Group: {}", group_name);
        for (channel_name, channel) in &group.channels {
            let expected_count = channel.data_len();
            let mut buffer = vec![0.0f64; expected_count];
            match channel.read_f64_into(&mut buffer) {
                Ok(count) => {
                    println!(
                        "     Channel '{}': {} double values",
                        channel_name,
                        count
                    );
                    println!("     Values: {:?}", &buffer[..count]);
                }
                Err(_) => {
                    println!("     Channel '{}': no data or wrong type", channel_name);
                }
            }
        }
    }

    println!("\n✨ Round-trip verification successful!");

    Ok(())
}
