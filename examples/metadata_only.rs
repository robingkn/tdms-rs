use tdms_rs::{PropertyValue, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsWriter::create("metadata_only.tdms")?;

    // File-level properties
    writer.add_property("Title", "Field Test".into())?;
    writer.add_property(
        "Author",
        PropertyValue::String("Rust Developer".to_string()),
    )?;
    writer.add_property("Version", 1.0f64.into())?;

    let mut group = writer.add_group("Sensors")?;
    group.add_property("Location", "Test Stand 4".into())?;

    let mut channel = group.add_channel::<f64>("Temperature")?;
    channel.add_property("Unit", "Celsius".into())?;
    channel.add_property("Calibrated", true.into())?;

    // Even metadata-heavy files need some data or close() to finalize
    channel.write(&[22.5])?;

    writer.close()?;
    println!("Metadata-rich file 'metadata_only.tdms' created.");

    Ok(())
}
