//! Demonstrates using waveform timing properties and the timestamps() iterator.
//! Shows how TDMS can store evenly-spaced time series and how to retrieve timestamps.

use std::path::Path;
use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("waveform_timestamps.tdms");

    // Write a waveform with timing properties
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Waveform")?;
        let mut channel = group.add_channel::<f64>("Signal")?;

        // Add standard waveform timing properties
        channel.add_property("wf_start_time", PropertyValue::Double(0.0))?; // seconds
        channel.add_property("wf_increment", PropertyValue::Double(0.001))?; // 1 ms per sample

        // Write some data (e.g., a simple sine wave)
        let data: Vec<f64> = (0..1000)
            .map(|i| (2.0 * std::f64::consts::PI * i as f64 * 0.01).sin())
            .collect();
        channel.write(&data)?;

        writer.close()?;
    }

    // Read back and demonstrate timestamp iteration
    let file = TdmsFile::open(path)?;
    let group = file.group("Waveform").unwrap();
    let channel = group.channel("Signal").unwrap();

    println!("Channel length: {}", channel.len());
    println!("Data type: {:?}", channel.dtype());

    // Verify timing properties
    match channel.property("wf_start_time") {
        Some(PropertyValue::Double(start)) => println!("Start time: {} s", start),
        _ => println!("No wf_start_time property"),
    }

    match channel.property("wf_increment") {
        Some(PropertyValue::Double(inc)) => println!("Increment: {} s", inc),
        _ => println!("No wf_increment property"),
    }

    // Use the timestamps iterator if timing properties are present
    if let Some(timestamps) = channel.timestamps() {
        println!("Timestamps available:");
        for (i, ts) in timestamps.take(10).enumerate() {
            println!("  Sample {}: {:.3} s", i, ts);
        }
        if channel.len() > 10 {
            println!("  ... (showing first 10 of {})", channel.len());
        }
    } else {
        println!("No timestamps iterator available (missing wf_start_time or wf_increment)");
    }

    // Read a few data points to verify
    let mut data = vec![0.0f64; 5];
    channel.read(0..5, &mut data)?;
    println!("First 5 data points: {:?}", data);

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
