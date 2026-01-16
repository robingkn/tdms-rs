//! Demonstrates safe abort of a TDMS write operation.
//! Shows how to abort and ensure no file is left behind.

use std::path::Path;
use tdms_rs::TdmsWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("abort_example.tdms");

    // Start a write operation that we will intentionally abort
    let mut writer = TdmsWriter::create(path)?;
    let mut group = writer.add_group("Temp")?;
    let mut channel = group.add_channel::<f64>("Data")?;
    channel.write(&[1.0, 2.0, 3.0])?;

    // At this point, the file does not exist yet (data is buffered)
    assert!(!path.exists(), "File should not exist before close()");

    // Simulate an error condition requiring abort
    let should_abort = true; // imagine this came from validation logic
    if should_abort {
        println!("Aborting write operation...");
        writer.abort()?; // This deletes the file if it exists
        assert!(!path.exists(), "File should not exist after abort");
        println!("Write aborted successfully; no file left behind.");
        return Ok(());
    }

    // Normal close path (not reached in this example)
    writer.close()?;
    assert!(path.exists(), "File should exist after close");
    std::fs::remove_file(path)?;
    Ok(())
}
