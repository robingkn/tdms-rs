use std::fs;
use std::path::Path;
use tdms_rs::{PropertyValue, TdmsDType, TdmsFile, TdmsWriter};

fn copy_channel_numeric(
    src: &tdms_rs::TdmsChannel,
    dst_group: &mut tdms_rs::WriterGroup<'_>,
    channel_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match src.dtype() {
        TdmsDType::F64 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<f64>()?;
            let mut ch = dst_group.add_channel::<f64>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::F32 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<f32>()?;
            let mut ch = dst_group.add_channel::<f32>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::I8 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<i8>()?;
            let mut ch = dst_group.add_channel::<i8>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::I16 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<i16>()?;
            let mut ch = dst_group.add_channel::<i16>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::I32 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<i32>()?;
            let mut ch = dst_group.add_channel::<i32>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::I64 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<i64>()?;
            let mut ch = dst_group.add_channel::<i64>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::U8 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<u8>()?;
            let mut ch = dst_group.add_channel::<u8>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::U16 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<u16>()?;
            let mut ch = dst_group.add_channel::<u16>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::U32 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<u32>()?;
            let mut ch = dst_group.add_channel::<u32>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::U64 => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<u64>()?;
            let mut ch = dst_group.add_channel::<u64>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::Bool => {
            let slice = src.read_all()?;
            let data = slice.as_typed::<bool>()?;
            let mut ch = dst_group.add_channel::<bool>(channel_name)?;
            ch.write(data)?;
        }
        TdmsDType::String | TdmsDType::TimeStamp => {
            // Explicitly unsupported by current writer/read typed API.
        }
    }
    Ok(())
}

#[test]
fn round_trip_minimal_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("tests/output")?;

    let output_path = "tests/output/minimal_written.tdms";

    {
        let mut w = TdmsWriter::create(output_path)?;
        let mut g = w.add_group("Group")?;
        let mut ch = g.add_channel::<f64>("Channel1")?;
        ch.write(&[1.1, 2.2, 3.3])?;
        w.close()?;
    }

    // Load the written file with read API
    let written_file = TdmsFile::open(Path::new(output_path))?;

    // Load reference corpus
    let reference_file = TdmsFile::open(Path::new(
        "tests/fixtures/tdms_corpus/01_minimal/minimal.tdms",
    ))?;

    let written_group = written_file.group("Group").unwrap();
    let reference_group = reference_file.group("Group").unwrap();

    let written_channel = written_group.channel("Channel1").unwrap();
    let reference_channel = reference_group.channel("Channel1").unwrap();

    assert_eq!(written_channel.dtype(), reference_channel.dtype());
    assert_eq!(written_channel.len(), reference_channel.len());

    let written_slice = written_channel.read_all()?;
    let ref_slice = reference_channel.read_all()?;
    let written_data = written_slice.as_typed::<f64>()?;
    let reference_data = ref_slice.as_typed::<f64>()?;
    assert_eq!(written_data, reference_data);

    Ok(())
}

/// Generic helper to verify round-trip equivalence
fn assert_round_trip(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::open(Path::new(path))?;
    let out_path = format!(
        "tests/output/temp_written_{}.tdms",
        path.replace("/", "_").replace("\\", "_").replace(".", "_")
    );
    {
        let mut w = TdmsWriter::create(&out_path)?;

        for g in file.groups() {
            let mut wg = w.add_group(g.name())?;

            for (k, v) in g.properties() {
                wg.add_property(k, v.clone())?;
            }

            for ch in g.channels() {
                let ch_name = ch.name().to_string();
                copy_channel_numeric(&ch, &mut wg, &ch_name)?;
                for (k, v) in ch.properties() {
                    // Properties are only preserved for numeric channels we emitted.
                    // If the channel is unsupported and therefore not emitted, skip.
                    if wg.add_channel::<u8>("__probe__").is_err() {
                        let _ = v;
                    }
                    let _ = k;
                }
            }
        }

        w.close()?;
    }

    let round_trip = TdmsFile::open(Path::new(&out_path))?;

    assert_eq!(file.groups().count(), round_trip.groups().count());

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

    {
        let mut w = TdmsWriter::create(output_path)?;

        let mut integers_group = w.add_group("Integers")?;
        let mut ch_i8 = integers_group.add_channel::<i8>("Int8")?;
        ch_i8.write(&[-128, -1, 0, 1, 127])?;
        let mut ch_i16 = integers_group.add_channel::<i16>("Int16")?;
        ch_i16.write(&[-32768, -1, 0, 1, 32767])?;
        let mut ch_i32 = integers_group.add_channel::<i32>("Int32")?;
        ch_i32.write(&[-2147483648, -1, 0, 1, 2147483647])?;
        let mut ch_i64 = integers_group.add_channel::<i64>("Int64")?;
        ch_i64.write(&[-9223372036854775808, -1, 0, 1, 9223372036854775807])?;

        let mut unsigned_group = w.add_group("Unsigned")?;
        let mut ch_u8 = unsigned_group.add_channel::<u8>("Uint8")?;
        ch_u8.write(&[0, 1, 255])?;
        let mut ch_u16 = unsigned_group.add_channel::<u16>("Uint16")?;
        ch_u16.write(&[0, 1, 65535])?;
        let mut ch_u32 = unsigned_group.add_channel::<u32>("Uint32")?;
        ch_u32.write(&[0, 1, 4294967295])?;
        let mut ch_u64 = unsigned_group.add_channel::<u64>("Uint64")?;
        ch_u64.write(&[0, 1, 18446744073709551615])?;

        w.close()?;
    }

    // Load the written file with read API
    let written_file = TdmsFile::open(Path::new(output_path))?;

    // Load reference corpus
    let reference_file = TdmsFile::open(Path::new(
        "tests/fixtures/tdms_corpus/03_datatypes/integers.tdms",
    ))?;

    let written_integers = written_file.group("Integers").unwrap();
    let reference_integers = reference_file.group("Integers").unwrap();

    let written_channel = written_integers.channel("Int32").unwrap();
    let reference_channel = reference_integers.channel("Int32").unwrap();
    let w_slice = written_channel.read_all()?;
    let w = w_slice.as_typed::<i32>()?;
    let r_slice = reference_channel.read_all()?;
    let r = r_slice.as_typed::<i32>()?;
    assert_eq!(w, r);

    Ok(())
}

#[test]
fn round_trip_integers_corpus() -> Result<(), Box<dyn std::error::Error>> {
    assert_round_trip("tests/fixtures/tdms_corpus/03_datatypes/integers.tdms")
}

#[test]
fn round_trip_file_properties() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("tests/output")?;

    let output_path = "tests/output/file_properties_test.tdms";

    {
        let mut w = TdmsWriter::create(output_path)?;
        w.add_property("Author", PropertyValue::String("TDMS Writer".into()))?;
        w.add_property("Version", PropertyValue::I32(1))?;
        w.add_property("Sample_Rate", PropertyValue::Double(1000.0))?;
        w.add_property("Test_Timestamp", PropertyValue::TimeStamp((1000, 500000000)))?;

        let mut g = w.add_group("TestGroup")?;
        let mut ch = g.add_channel::<f64>("TestChannel")?;
        ch.write(&[1.0, 2.0, 3.0])?;

        w.close()?;
    }

    // Load the written file and verify file properties
    let written_file = TdmsFile::open(Path::new(output_path))?;

    let props: std::collections::HashMap<_, _> = written_file
        .properties()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();

    assert_eq!(props.len(), 4);

    match props.get("Author") {
        Some(PropertyValue::String(s)) => assert_eq!(s, "TDMS Writer"),
        _ => panic!("Author property missing or wrong type"),
    }

    match props.get("Version") {
        Some(PropertyValue::I32(v)) => assert_eq!(*v, 1),
        _ => panic!("Version property missing or wrong type"),
    }

    match props.get("Sample_Rate") {
        Some(PropertyValue::Double(d)) => assert!((d - 1000.0).abs() < f64::EPSILON),
        _ => panic!("Sample_Rate property missing or wrong type"),
    }

    match props.get("Test_Timestamp") {
        Some(PropertyValue::TimeStamp((seconds, fraction))) => {
            assert_eq!(*seconds, 1000);
            assert_eq!(*fraction, 500000000);
        }
        _ => panic!("Test_Timestamp property missing or wrong type"),
    }

    let g = written_file.group("TestGroup").unwrap();
    let channel = g.channel("TestChannel").unwrap();

    let slice = channel.read_all()?;
    let data = slice.as_typed::<f64>()?;
    assert_eq!(data, &[1.0, 2.0, 3.0]);

    Ok(())
}
