//! Minimal write+read example for f64 channel data.
//! Demonstrates creating a file, writing a channel, then reading it back.

use std::path::Path;
use tdms_rs::{TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("basic_f64.tdms");

    // Write a simple TDMS file with one f64 channel
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Sensors")?;
        let mut channel = group.add_channel::<f64>("Temperature")?;
        channel.write(&[20.0, 21.0, 22.0, 23.0, 24.0])?;
        writer.close()?;
    }

    // Read the file back and verify
    let file = TdmsFile::open(path)?;
    let group = file.group("Sensors").unwrap();
    let channel = group.channel("Temperature").unwrap();

    assert_eq!(channel.len(), 5);
    let mut data = vec![0.0f64; channel.len()];
    channel.read(0..channel.len(), &mut data)?;
    assert_eq!(data, &[20.0, 21.0, 22.0, 23.0, 24.0]);

    println!("Read {} values: {:?}", data.len(), data);

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
