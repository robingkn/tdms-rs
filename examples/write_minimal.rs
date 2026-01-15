//! Minimal TDMS file writing example.
//!
//! This example demonstrates the simplest way to create a TDMS file
//! with a single group and channel containing double-precision data.

use std::fs;
use tdms_rs::{TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating minimal TDMS file...");

    // Create a new TDMS file writer
    let mut writer = TdmsWriter::create("examples/output/minimal.tdms")?;

    // Add a group
    let mut group = writer.add_group("Measurements")?;

    // Add a channel with some sample data
    let mut ch = group.add_channel::<f64>("Temperature")?;
    ch.write(&[20.1, 21.5, 22.3, 23.0, 22.8])?;

    // Write the file
    writer.close()?;

    println!("✅ Successfully created 'examples/output/minimal.tdms'");
    println!("   - 1 group: 'Measurements'");
    println!("   - 1 channel: 'Temperature' with 5 double values");

    // Verify by reading it back
    println!("\n🔍 Verifying by reading the file back...");
    let file = TdmsFile::open(std::path::Path::new("examples/output/minimal.tdms"))?;

    for g in file.groups() {
        println!("   Group: {}", g.name());
        for c in g.channels() {
            if c.dtype() == tdms_rs::TdmsDType::F64 {
                let slice = c.read_all()?;
                let data = slice.as_typed::<f64>()?;
                println!("     Channel '{}': {} double values", c.name(), data.len());
                println!("     Values: {:?}", data);
            } else {
                println!("     Channel '{}': {} samples ({:?})", c.name(), c.len(), c.dtype());
            }
        }
    }

    println!("\n✨ Round-trip verification successful!");

    Ok(())
}
