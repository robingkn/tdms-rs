use tdms_rs::TdmsWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a new TDMS file
    let mut writer = TdmsWriter::create("append_example.tdms")?;

    // 2. Build the structure
    let mut group = writer.add_group("Engine")?;
    let mut rpm = group.add_channel::<f64>("RPM")?;

    // 3. Initial write
    println!("Writing initial batch...");
    rpm.write(&[1000.0, 1100.0, 1200.0])?;

    // 4. "Append" by calling write again on the same channel
    // In the current implementation, all data is batched until close()
    println!("Appending more data...");
    rpm.write(&[1300.0, 1400.0, 1500.0])?;

    // 5. Finalize
    writer.close()?;
    println!("File 'append_example.tdms' created successfully.");

    Ok(())
}
