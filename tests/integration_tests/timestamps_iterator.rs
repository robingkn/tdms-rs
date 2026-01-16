//! Tests the timestamps() iterator for waveform timing properties.
//! Verifies correct behavior when wf_start_time and wf_increment are present/absent.

use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

#[test]
fn timestamps_iterator_present() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/timestamps_present.tdms";
    std::fs::create_dir_all("tests/output")?;

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Wave")?;
        let mut channel = group.add_channel::<f64>("Signal")?;

        channel.add_property("wf_start_time", PropertyValue::Double(100.0))?;
        channel.add_property("wf_increment", PropertyValue::Double(0.1))?;

        channel.write(&[1.0, 2.0, 3.0, 4.0, 5.0])?;
        writer.close()?;
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("Wave").unwrap().channel("Signal").unwrap();

    // Timestamps should be available
    let timestamps = channel.timestamps().unwrap();
    let collected: Vec<f64> = timestamps.collect();

    assert_eq!(collected.len(), 5);
    assert_eq!(collected[0], 100.0); // start_time
    assert_eq!(collected[1], 100.1); // start_time + increment
    assert_eq!(collected[2], 100.2);
    assert_eq!(collected[3], 100.3);
    assert_eq!(collected[4], 100.4);

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn timestamps_iterator_missing_properties() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/timestamps_missing.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Wave")?;
        let mut channel = group.add_channel::<f64>("Signal")?;

        // No wf_start_time or wf_increment properties
        channel.write(&[1.0, 2.0, 3.0])?;
        writer.close()?;
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("Wave").unwrap().channel("Signal").unwrap();

    // Timestamps should not be available
    assert!(channel.timestamps().is_none());

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn timestamps_iterator_partial_properties() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/timestamps_partial.tdms";

    // Test with only wf_start_time (missing wf_increment)
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Wave")?;
        let mut channel = group.add_channel::<f64>("Signal")?;

        channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
        // Missing wf_increment

        channel.write(&[1.0, 2.0])?;
        writer.close()?;
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("Wave").unwrap().channel("Signal").unwrap();

    // Should not have timestamps without both properties
    assert!(channel.timestamps().is_none());

    std::fs::remove_file(path)?;

    // Test with only wf_increment (missing wf_start_time)
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Wave")?;
        let mut channel = group.add_channel::<f64>("Signal")?;

        // Missing wf_start_time
        channel.add_property("wf_increment", PropertyValue::Double(0.5))?;

        channel.write(&[1.0, 2.0])?;
        writer.close()?;
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("Wave").unwrap().channel("Signal").unwrap();

    // Should not have timestamps without both properties
    assert!(channel.timestamps().is_none());

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn timestamps_iterator_empty_channel() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/timestamps_empty.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Wave")?;
        let mut channel = group.add_channel::<f64>("Signal")?;

        channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
        channel.add_property("wf_increment", PropertyValue::Double(1.0))?;

        // No data written
        writer.close()?;
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("Wave").unwrap().channel("Signal").unwrap();

    // Timestamps should be available but empty
    let timestamps = channel.timestamps().unwrap();
    let collected: Vec<f64> = timestamps.collect();
    assert_eq!(collected.len(), 0);

    std::fs::remove_file(path)?;
    Ok(())
}
