//! Smoke test: write a simple file and read it back.
//! Covers basic writer/reader workflow without external dependencies.

use tdms_rs::{TdmsFile, TdmsWriter};

#[test]
fn smoke_roundtrip_f64() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/smoke_roundtrip_f64.tdms";
    std::fs::create_dir_all("tests/output")?;

    // Write
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Test")?;
        let mut channel = group.add_channel::<f64>("Data")?;
        channel.write(&[1.0, 2.0, 3.0, 4.0, 5.0])?;
        writer.close()?;
    }

    // Read and verify
    let file = TdmsFile::open(path)?;
    let group = file.group("Test").unwrap();
    let channel = group.channel("Data").unwrap();

    assert_eq!(channel.len(), 5);
    assert_eq!(channel.dtype(), tdms_rs::DataType::Double);

    let mut data = vec![0.0f64; 5];
    channel.read(0..5, &mut data)?;
    assert_eq!(data, &[1.0, 2.0, 3.0, 4.0, 5.0]);

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn smoke_roundtrip_bool() -> Result<(), Box<dyn std::error::Error>> {
    let path = "tests/output/smoke_roundtrip_bool.tdms";

    // Write
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Test")?;
        let mut channel = group.add_channel::<bool>("Flags")?;
        channel.write(&[true, false, true, false, true])?;
        writer.close()?;
    }

    // Read and verify
    let file = TdmsFile::open(path)?;
    let channel = file.group("Test").unwrap().channel("Flags").unwrap();

    assert_eq!(channel.len(), 5);
    assert_eq!(channel.dtype(), tdms_rs::DataType::Boolean);

    let mut data = vec![false; 5];
    channel.read(0..5, &mut data)?;
    assert_eq!(data, &[true, false, true, false, true]);

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
