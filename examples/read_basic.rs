use std::path::Path;
use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open the TDMS file
    // TdmsFile::open indexes the file structure (groups, channels, properties)
    // without loading the raw data into memory.
    let file = TdmsFile::open(Path::new("data.tdms"))?;

    // 2. Navigate the hierarchy
    // Access groups and channels by name.
    if let Some(group) = file.group("Sensors") {
        println!("Group: {}", group.name());

        if let Some(channel) = group.channel("Temperature") {
            println!("  Channel: {} ({} samples)", channel.name(), channel.len());

            // 3. Read data
            // read_into() loads the data into a pre-allocated buffer.
            let mut data = vec![0.0f64; channel.len()];
            channel.read_into(0..channel.len(), &mut data)?;

            if !data.is_empty() {
                let avg: f64 = data.iter().sum::<f64>() / data.len() as f64;
                println!("  Average Temperature: {:.2}°C", avg);
            }
        }
    }

    Ok(())
}
