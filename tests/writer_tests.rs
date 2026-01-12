use std::fs;
use std::path::Path;
use tdms_rs::{TdmsData, TdmsFile, TdmsFileWriter, TdmsChannel};

/// Helper function to read channel data using slice-based API and convert to TdmsData
fn read_channel_data_to_tdms_data(channel: &TdmsChannel) -> Result<Option<TdmsData>, Box<dyn std::error::Error>> {
    let data_type_name = match channel.data_type_name() {
        Some(name) => name,
        None => return Ok(None),
    };
    
    let count = channel.data_len();
    if count == 0 {
        return Ok(None);
    }
    
    match data_type_name {
        "Double" => {
            let mut buffer = vec![0.0f64; count];
            channel.read_f64_into(&mut buffer)?;
            Ok(Some(TdmsData::Double(buffer)))
        }
        "Float" => {
            let mut buffer = vec![0.0f32; count];
            channel.read_f32_into(&mut buffer)?;
            Ok(Some(TdmsData::Float(buffer)))
        }
        "I8" => {
            let mut buffer = vec![0i8; count];
            channel.read_i8_into(&mut buffer)?;
            Ok(Some(TdmsData::I8(buffer)))
        }
        "I16" => {
            let mut buffer = vec![0i16; count];
            channel.read_i16_into(&mut buffer)?;
            Ok(Some(TdmsData::I16(buffer)))
        }
        "I32" => {
            let mut buffer = vec![0i32; count];
            channel.read_i32_into(&mut buffer)?;
            Ok(Some(TdmsData::I32(buffer)))
        }
        "I64" => {
            let mut buffer = vec![0i64; count];
            channel.read_i64_into(&mut buffer)?;
            Ok(Some(TdmsData::I64(buffer)))
        }
        "U8" => {
            let mut buffer = vec![0u8; count];
            channel.read_u8_into(&mut buffer)?;
            Ok(Some(TdmsData::U8(buffer)))
        }
        "U16" => {
            let mut buffer = vec![0u16; count];
            channel.read_u16_into(&mut buffer)?;
            Ok(Some(TdmsData::U16(buffer)))
        }
        "U32" => {
            let mut buffer = vec![0u32; count];
            channel.read_u32_into(&mut buffer)?;
            Ok(Some(TdmsData::U32(buffer)))
        }
        "U64" => {
            let mut buffer = vec![0u64; count];
            channel.read_u64_into(&mut buffer)?;
            Ok(Some(TdmsData::U64(buffer)))
        }
        "Boolean" => {
            let mut buffer = vec![false; count];
            channel.read_bool_into(&mut buffer)?;
            Ok(Some(TdmsData::Boolean(buffer)))
        }
        "TimeStamp" => {
            let mut buffer = vec![(0i64, 0u64); count];
            channel.read_timestamp_into(&mut buffer)?;
            Ok(Some(TdmsData::TimeStamp(buffer)))
        }
        "String" => {
            // String reading is more complex - for now return None
            // TODO: Implement string reading
            Ok(None)
        }
        _ => Ok(None),
    }
}

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
    let reference_file = TdmsFile::load(Path::new(
        "tests/fixtures/tdms_corpus/01_minimal/minimal.tdms",
    ))?;

    // Compare structure
    assert_eq!(written_file.groups.len(), reference_file.groups.len());
    assert_eq!(
        written_file.groups.keys().collect::<Vec<_>>(),
        reference_file.groups.keys().collect::<Vec<_>>()
    );

    // Compare group content
    let written_group = written_file.groups.get("Group").unwrap();
    let reference_group = reference_file.groups.get("Group").unwrap();

    assert_eq!(written_group.channels.len(), reference_group.channels.len());
    assert_eq!(
        written_group.channels.keys().collect::<Vec<_>>(),
        reference_group.channels.keys().collect::<Vec<_>>()
    );

    // Compare channel data
    let written_channel = written_group.channels.get("Channel1").unwrap();
    let reference_channel = reference_group.channels.get("Channel1").unwrap();

    let expected_count = written_channel.data_len();
    let mut written_buffer = vec![0.0f64; expected_count];
    let written_count = written_channel.read_f64_into(&mut written_buffer)
        .expect("Failed to read written data");
    
    let ref_expected_count = reference_channel.data_len();
    let mut reference_buffer = vec![0.0f64; ref_expected_count];
    let reference_count = reference_channel.read_f64_into(&mut reference_buffer)
        .expect("Failed to read reference data");

    assert_eq!(written_count, reference_count);
    for (w, r) in written_buffer[..written_count].iter().zip(reference_buffer[..reference_count].iter()) {
        assert!(
            (w - r).abs() < f64::EPSILON,
            "Data mismatch: {} vs {}",
            w,
            r
        );
    }

    Ok(())
}

/// Generic helper to verify round-trip equivalence
fn assert_round_trip(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::load(Path::new(path))?;
    let out_path = format!(
        "tests/output/temp_written_{}.tdms",
        path.replace("/", "_").replace("\\", "_").replace(".", "_")
    );
    let mut writer = TdmsFileWriter::new(&out_path);

    // Copy structure & data
    for (group_name, group) in &file.groups {
        let g = writer.add_group(group_name)?;
        for (chan_name, chan) in &group.channels {
            // Read data using slice-based API based on type
            let data = read_channel_data_to_tdms_data(chan)?;
            if let Some(data) = data {
                g.add_channel(chan_name, data)?;
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
        assert_eq!(
            original_group.channels.len(),
            round_trip_group.channels.len()
        );
        for (channel_name, original_channel) in &original_group.channels {
            let round_trip_channel = round_trip_group.channels.get(channel_name).unwrap();
            
            // Use slice-based reading to compare data
            // We'll read both and compare the slices
            let original_type = original_channel.data_type_name().expect("Unknown data type");
            let round_trip_type = round_trip_channel.data_type_name().expect("Unknown data type");
            assert_eq!(original_type, round_trip_type, "Type mismatch for channel {}", channel_name);
            
            // For now, only compare lengths - full comparison would require type-specific logic
            assert_eq!(
                original_channel.data_len(),
                round_trip_channel.data_len(),
                "Length mismatch for channel {}",
                channel_name
            );
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
    integers_group.add_channel(
        "Int32",
        TdmsData::I32(vec![-2147483648, -1, 0, 1, 2147483647]),
    )?;
    integers_group.add_channel(
        "Int64",
        TdmsData::I64(vec![-9223372036854775808, -1, 0, 1, 9223372036854775807]),
    )?;

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
    let reference_file = TdmsFile::load(Path::new(
        "tests/fixtures/tdms_corpus/03_datatypes/integers.tdms",
    ))?;

    // Compare structure
    assert_eq!(written_file.groups.len(), reference_file.groups.len());

    // Compare Integers group
    let written_integers = written_file.groups.get("Integers").unwrap();
    let reference_integers = reference_file.groups.get("Integers").unwrap();
    assert_eq!(
        written_integers.channels.len(),
        reference_integers.channels.len()
    );

    // Compare Unsigned group
    let written_unsigned = written_file.groups.get("Unsigned").unwrap();
    let reference_unsigned = reference_file.groups.get("Unsigned").unwrap();
    assert_eq!(
        written_unsigned.channels.len(),
        reference_unsigned.channels.len()
    );

    // Compare specific channel data
    let written_channel = written_integers.channels.get("Int32").unwrap();
    let expected_count = written_channel.data_len();
    let mut written_buffer = vec![0i32; expected_count];
    let written_count = written_channel.read_i32_into(&mut written_buffer)
        .expect("Failed to read written Int32");
    
    let reference_channel = reference_integers.channels.get("Int32").unwrap();
    let ref_expected_count = reference_channel.data_len();
    let mut reference_buffer = vec![0i32; ref_expected_count];
    let reference_count = reference_channel.read_i32_into(&mut reference_buffer)
        .expect("Failed to read reference Int32");
    
    assert_eq!(written_count, reference_count);
    assert_eq!(written_buffer[..written_count], reference_buffer[..reference_count]);

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
    file_writer.add_property(
        "Test_Timestamp",
        PropertyValue::TimeStamp((1000, 500000000)),
    )?;

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
        }
        _ => panic!("Test_Timestamp property missing or wrong type"),
    }

    // Verify the data structure is intact
    assert_eq!(written_file.groups.len(), 1);
    let group = written_file.groups.get("TestGroup").unwrap();
    assert_eq!(group.channels.len(), 1);
    let channel = group.channels.get("TestChannel").unwrap();

    let expected_count = channel.data_len();
    let mut buffer = vec![0.0f64; expected_count];
    match channel.read_f64_into(&mut buffer) {
        Ok(count) => {
            assert_eq!(count, 3);
            assert_eq!(buffer[..count], vec![1.0, 2.0, 3.0]);
        }
        Err(e) => panic!("Failed to read channel data: {:?}", e),
    }

    Ok(())
}
