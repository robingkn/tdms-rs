//! Tests error handling for invalid read ranges and buffer sizes.
//! Ensures the reader API validates inputs correctly.

use tdms_rs::{TdmsFile, TdmsWriter};

#[test]
fn read_invalid_range() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/read_invalid_range.tdms";
    std::fs::create_dir_all("tests/output")?;

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let mut channel = group.add_channel::<f64>("C")?;
        channel.write(&[1.0, 2.0, 3.0])?;
        // File is automatically flushed and closed when writer goes out of scope
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("C").unwrap();

    // Test various invalid ranges
    let mut buf = [0.0f64; 10];

    // Start beyond end
    let result = channel.read(5..8, &mut buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        tdms_rs::TdmsError::InvalidRange(start, end, len) => {
            assert_eq!(start, 5);
            assert_eq!(end, 8);
            assert_eq!(len, 3);
        }
        _ => panic!("Expected InvalidRange error"),
    }

    // End beyond channel length
    let result = channel.read(1..10, &mut buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        tdms_rs::TdmsError::InvalidRange(start, end, len) => {
            assert_eq!(start, 1);
            assert_eq!(end, 10);
            assert_eq!(len, 3);
        }
        _ => panic!("Expected InvalidRange error"),
    }

    // Empty range (should be valid)
    let result = channel.read(2..2, &mut buf[0..0]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn read_buffer_too_small() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/read_buffer_small.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let mut channel = group.add_channel::<f64>("C")?;
        channel.write(&[1.0, 2.0, 3.0])?;
        // File is automatically flushed and closed when writer goes out of scope
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("C").unwrap();

    // Buffer smaller than requested range
    let mut buf = [0.0f64; 1]; // Only space for 1 element, requesting 2
    let result = channel.read(0..2, &mut buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        tdms_rs::TdmsError::InvalidFormat(msg) => {
            assert!(msg.contains("output buffer too small"));
        }
        _ => panic!("Expected InvalidFormat error for buffer too small"),
    }

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn read_type_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/read_type_mismatch.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let mut channel = group.add_channel::<f64>("C")?;
        channel.write(&[1.0, 2.0, 3.0])?;
        // File is automatically flushed and closed when writer goes out of scope
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("C").unwrap();

    // Try to read f64 data into i32 buffer
    let mut buf = [0i32; 3];
    let result = channel.read(0..3, &mut buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        tdms_rs::TdmsError::TypeMismatch => {} // Expected
        _ => panic!("Expected TypeMismatch error"),
    }

    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn read_empty_channel() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/read_empty.tdms";

    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let _channel = group.add_channel::<f64>("C")?;
        // Write no data
        // File is automatically flushed and closed when writer goes out of scope
    }

    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("C").unwrap();

    assert_eq!(channel.len(), 0);
    assert!(channel.is_empty());

    // Reading from empty channel should work for empty range
    let mut buf: [f64; 0] = [];
    let result = channel.read(0..0, &mut buf);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    // Any non-empty range should fail
    let mut buf = [0.0f64];
    let result = channel.read(0..1, &mut buf);
    assert!(result.is_err());

    std::fs::remove_file(path)?;
    Ok(())
}
