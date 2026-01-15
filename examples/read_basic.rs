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
            // read_all() loads the entire channel data into a TdmsSlice.
            let slice = channel.read_all()?;

            // 4. Access as typed slice
            // as_typed() provides zero-copy access to the underlying buffer
            // if the type matches (e.g., f64 for a double channel).
            let data: &[f64] = slice.as_typed()?;

            if !data.is_empty() {
                let avg = data.iter().sum::<f64>() / data.len() as f64;
                println!("  Average Temperature: {:.2}°C", avg);
            }
        }
    }

    Ok(())
}
