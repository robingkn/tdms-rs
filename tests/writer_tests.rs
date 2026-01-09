use tdms_rs::{TdmsFile, TdmsFileWriter, TdmsData};
use std::fs;
use std::path::Path;

#[test]
fn round_trip_minimal_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("tests/output")?;
    
    let output_path = "tests/output/minimal_written.tdms";
    
    // Build file using writer API to match minimal.tdms
    let mut file_writer = TdmsFileWriter::new(output_path);
    let group = file_writer.add_group("Group")?;
    group.add_channel("Channel1", TdmsData::Double(vec![1.1, 2.2, 3.3]))?;
    file_writer.write()?;
    
    // Load the written file with read API
    let written_file = TdmsFile::load(Path::new(output_path))?;
    
    // Load reference corpus
    let reference_file = TdmsFile::load(Path::new("tests/fixtures/tdms_corpus/01_minimal/minimal.tdms"))?;
    
    // Compare structure
    assert_eq!(written_file.groups.len(), reference_file.groups.len());
    assert_eq!(written_file.groups.keys().collect::<Vec<_>>(), reference_file.groups.keys().collect::<Vec<_>>());
    
    // Compare group content
    let written_group = written_file.groups.get("Group").unwrap();
    let reference_group = reference_file.groups.get("Group").unwrap();
    
    assert_eq!(written_group.channels.len(), reference_group.channels.len());
    assert_eq!(written_group.channels.keys().collect::<Vec<_>>(), reference_group.channels.keys().collect::<Vec<_>>());
    
    // Compare channel data
    let written_channel = written_group.channels.get("Channel1").unwrap();
    let reference_channel = reference_group.channels.get("Channel1").unwrap();
    
    match (&written_channel.data, &reference_channel.data) {
        (Some(TdmsData::Double(written_data)), Some(TdmsData::Double(reference_data))) => {
            assert_eq!(written_data.len(), reference_data.len());
            for (w, r) in written_data.iter().zip(reference_data.iter()) {
                assert!((w - r).abs() < f64::EPSILON, "Data mismatch: {} vs {}", w, r);
            }
        },
        _ => panic!("Data type mismatch or missing data"),
    }
    
    Ok(())
}

/// Generic helper to verify round-trip equivalence
fn assert_round_trip(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::load(Path::new(path))?;
    let out_path = format!("tests/output/temp_written_{}.tdms", path.replace("/", "_").replace("\\", "_").replace(".", "_"));
    let mut writer = TdmsFileWriter::new(&out_path);
    
    // Copy structure & data
    for (group_name, group) in &file.groups {
        let g = writer.add_group(group_name)?;
        for (chan_name, chan) in &group.channels {
            if let Some(data) = &chan.data {
                g.add_channel(chan_name, data.clone())?;
            }
        }
        for (k, v) in &group.properties {
            g.add_property(k, v.clone())?;
        }
    }
    writer.write()?;
    
    let round_trip = TdmsFile::load(Path::new(&out_path))?;
    
    // Compare groups
    assert_eq!(file.groups.len(), round_trip.groups.len());
    for (group_name, original_group) in &file.groups {
        let round_trip_group = round_trip.groups.get(group_name).unwrap();
        
        // Compare channels
        assert_eq!(original_group.channels.len(), round_trip_group.channels.len());
        for (channel_name, original_channel) in &original_group.channels {
            let round_trip_channel = round_trip_group.channels.get(channel_name).unwrap();
            assert_eq!(original_channel.data, round_trip_channel.data);
        }
    }
    
    Ok(())
}

#[test]
fn round_trip_minimal_corpus() -> Result<(), Box<dyn std::error::Error>> {
    assert_round_trip("tests/fixtures/tdms_corpus/01_minimal/minimal.tdms")
}

#[test]
fn round_trip_integers() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("tests/output")?;
    
    let output_path = "tests/output/integers_written.tdms";
    
    // Build file using writer API to match integers.tdms
    let mut file_writer = TdmsFileWriter::new(output_path);
    
    // Add Integers group
    let integers_group = file_writer.add_group("Integers")?;
    integers_group.add_channel("Int8", TdmsData::I8(vec![-128, -1, 0, 1, 127]))?;
    integers_group.add_channel("Int16", TdmsData::I16(vec![-32768, -1, 0, 1, 32767]))?;
    integers_group.add_channel("Int32", TdmsData::I32(vec![-2147483648, -1, 0, 1, 2147483647]))?;
    integers_group.add_channel("Int64", TdmsData::I64(vec![-9223372036854775808, -1, 0, 1, 9223372036854775807]))?;
    
    // Add Unsigned group
    let unsigned_group = file_writer.add_group("Unsigned")?;
    unsigned_group.add_channel("Uint8", TdmsData::U8(vec![0, 1, 255]))?;
    unsigned_group.add_channel("Uint16", TdmsData::U16(vec![0, 1, 65535]))?;
    unsigned_group.add_channel("Uint32", TdmsData::U32(vec![0, 1, 4294967295]))?;
    unsigned_group.add_channel("Uint64", TdmsData::U64(vec![0, 1, 18446744073709551615]))?;
    
    file_writer.write()?;
    
    // Load the written file with read API
    let written_file = TdmsFile::load(Path::new(output_path))?;
    
    // Load reference corpus
    let reference_file = TdmsFile::load(Path::new("tests/fixtures/tdms_corpus/03_datatypes/integers.tdms"))?;
    
    // Compare structure
    assert_eq!(written_file.groups.len(), reference_file.groups.len());
    
    // Compare Integers group
    let written_integers = written_file.groups.get("Integers").unwrap();
    let reference_integers = reference_file.groups.get("Integers").unwrap();
    assert_eq!(written_integers.channels.len(), reference_integers.channels.len());
    
    // Compare Unsigned group
    let written_unsigned = written_file.groups.get("Unsigned").unwrap();
    let reference_unsigned = reference_file.groups.get("Unsigned").unwrap();
    assert_eq!(written_unsigned.channels.len(), reference_unsigned.channels.len());
    
    // Compare specific channel data
    match (&written_integers.channels.get("Int32").unwrap().data, &reference_integers.channels.get("Int32").unwrap().data) {
        (Some(TdmsData::I32(written_data)), Some(TdmsData::I32(reference_data))) => {
            assert_eq!(written_data, reference_data);
        },
        _ => panic!("Int32 data type mismatch or missing data"),
    }
    
    Ok(())
}

#[test]
fn round_trip_integers_corpus() -> Result<(), Box<dyn std::error::Error>> {
    assert_round_trip("tests/fixtures/tdms_corpus/03_datatypes/integers.tdms")
}
#[test]
fn round_trip_file_properties() -> Result<(), Box<dyn std::error::Error>> {
    use tdms_rs::PropertyValue;
    
    // Create output directory if it doesn't exist
    fs::create_dir_all("tests/output")?;
    
    let output_path = "tests/output/file_properties_test.tdms";
    
    // Build file with file-level properties
    let mut file_writer = TdmsFileWriter::new(output_path);
    
    // Add file-level properties
    file_writer.add_property("Author", PropertyValue::String("TDMS Writer".into()))?;
    file_writer.add_property("Version", PropertyValue::I32(1))?;
    file_writer.add_property("Sample_Rate", PropertyValue::Double(1000.0))?;
    file_writer.add_property("Test_Timestamp", PropertyValue::TimeStamp((1000, 500000000)))?;
    
    // Add minimal data structure
    let group = file_writer.add_group("TestGroup")?;
    group.add_channel("TestChannel", TdmsData::Double(vec![1.0, 2.0, 3.0]))?;
    
    file_writer.write()?;
    
    // Load the written file and verify file properties
    let written_file = TdmsFile::load(Path::new(output_path))?;
    
    // Verify file properties exist and match
    assert_eq!(written_file.properties.len(), 4);
    
    match written_file.properties.get("Author") {
        Some(PropertyValue::String(s)) => assert_eq!(s, "TDMS Writer"),
        _ => panic!("Author property missing or wrong type"),
    }
    
    match written_file.properties.get("Version") {
        Some(PropertyValue::I32(v)) => assert_eq!(*v, 1),
        _ => panic!("Version property missing or wrong type"),
    }
    
    match written_file.properties.get("Sample_Rate") {
        Some(PropertyValue::Double(d)) => assert!((d - 1000.0).abs() < f64::EPSILON),
        _ => panic!("Sample_Rate property missing or wrong type"),
    }
    
    match written_file.properties.get("Test_Timestamp") {
        Some(PropertyValue::TimeStamp((seconds, fraction))) => {
            assert_eq!(*seconds, 1000);
            assert_eq!(*fraction, 500000000);
        },
        _ => panic!("Test_Timestamp property missing or wrong type"),
    }
    
    // Verify the data structure is intact
    assert_eq!(written_file.groups.len(), 1);
    let group = written_file.groups.get("TestGroup").unwrap();
    assert_eq!(group.channels.len(), 1);
    let channel = group.channels.get("TestChannel").unwrap();
    
    match &channel.data {
        Some(TdmsData::Double(data)) => {
            assert_eq!(data, &vec![1.0, 2.0, 3.0]);
        },
        _ => panic!("Channel data missing or wrong type"),
    }
    
    Ok(())
}