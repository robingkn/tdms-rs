use std::path::Path;
use tdms_rs::{TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("data.tdms");

    // 1. Create a sample TDMS file for this example
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Sensors")?;
        let mut channel = group.add_channel::<f64>("Temperature")?;
        // Some sample data
        channel.write(&[20.0, 21.0, 22.0, 23.0, 24.0])?;
        writer.close()?;
    }

    // 2. Open the TDMS file
    // TdmsFile::open indexes the file structure (groups, channels, properties)
    // without loading the raw data into memory.
    let file = TdmsFile::open(path)?;

    // 3. Navigate the hierarchy
    // Access groups and channels by name.
    if let Some(group) = file.group("Sensors") {
        println!("Group: {}", group.name());

        if let Some(channel) = group.channel("Temperature") {
            println!("  Channel: {} ({} samples)", channel.name(), channel.len());

            // 4. Read data
            // read() loads the data into a pre-allocated buffer.
            let mut data = vec![0.0f64; channel.len()];
            channel.read(0..channel.len(), &mut data)?;

            if !data.is_empty() {
                let avg: f64 = data.iter().sum::<f64>() / data.len() as f64;
                println!("  Average Temperature: {:.2}°C", avg);
            }
        }
    }

    // Clean up
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}
