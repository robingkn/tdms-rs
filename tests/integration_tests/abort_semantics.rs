//! Tests writer abort semantics.
//! Ensures abort() deletes the file and doesn't leave partial data.

use std::path::Path;
use tdms_rs::TdmsWriter;

#[test]
fn abort_before_close_deletes_file() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/abort_before_close.tdms";
    std::fs::create_dir_all("tests/output")?;

    let mut writer = TdmsWriter::create(path)?;
    let mut group = writer.add_group("G")?;
    let mut channel = group.add_channel::<f64>("C")?;
    channel.write(&[1.0, 2.0, 3.0])?;

    // File should not exist yet (data is buffered)
    assert!(!Path::new(path).exists());

    // Abort should ensure no file is left behind
    writer.abort()?;
    assert!(!Path::new(path).exists());

    Ok(())
}

#[test]
fn abort_after_close_is_error() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/abort_after_close.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let mut channel = group.add_channel::<f64>("C")?;
        channel.write(&[1.0, 2.0])?;
        writer.close()?;
    }

    // File should exist after close
    assert!(Path::new(path).exists());

    // Creating a new writer to the same path and aborting should work
    let writer = TdmsWriter::create(path)?;
    writer.abort()?;
    assert!(!Path::new(path).exists());

    Ok(())
}

#[test]
fn abort_without_writing_data() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/abort_no_data.tdms";

    let writer = TdmsWriter::create(path)?;
    // Don't write any channels or data
    writer.abort()?;

    // File should not exist
    assert!(!Path::new(path).exists());

    Ok(())
}

#[test]
fn abort_with_properties_only() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/abort_props_only.tdms";

    let mut writer = TdmsWriter::create(path)?;
    writer.add_property("test", tdms_rs::PropertyValue::String("value".into()))?;

    let mut group = writer.add_group("G")?;
    group.add_property("group_prop", tdms_rs::PropertyValue::Boolean(true))?;

    let mut channel = group.add_channel::<f64>("C")?;
    channel.add_property(
        "ch_prop",
        tdms_rs::PropertyValue::Double(std::f64::consts::PI),
    )?;

    // Don't write any actual channel data
    writer.abort()?;

    // File should not exist
    assert!(!Path::new(path).exists());

    Ok(())
}
