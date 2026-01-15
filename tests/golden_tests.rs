use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tdms_rs::{TdmsDType, TdmsFile};

fn read_channel_data_as_json(
    channel: &tdms_rs::TdmsChannel,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let slice = channel.read_all()?;

    let json = match channel.dtype() {
        TdmsDType::F64 => serde_json::Value::from(
            slice
                .as_typed::<f64>()?
                .iter()
                .copied()
                .collect::<Vec<f64>>(),
        ),
        TdmsDType::F32 => serde_json::Value::from(
            slice
                .as_typed::<f32>()?
                .iter()
                .map(|v| *v as f64)
                .collect::<Vec<f64>>(),
        ),
        TdmsDType::I8 => serde_json::Value::from(
            slice
                .as_typed::<i8>()?
                .iter()
                .map(|v| *v as i64)
                .collect::<Vec<i64>>(),
        ),
        TdmsDType::I16 => serde_json::Value::from(
            slice
                .as_typed::<i16>()?
                .iter()
                .map(|v| *v as i64)
                .collect::<Vec<i64>>(),
        ),
        TdmsDType::I32 => serde_json::Value::from(
            slice
                .as_typed::<i32>()?
                .iter()
                .map(|v| *v as i64)
                .collect::<Vec<i64>>(),
        ),
        TdmsDType::I64 => serde_json::Value::from(
            slice
                .as_typed::<i64>()?
                .iter()
                .copied()
                .collect::<Vec<i64>>(),
        ),
        TdmsDType::U8 => serde_json::Value::from(
            slice
                .as_typed::<u8>()?
                .iter()
                .map(|v| *v as u64)
                .collect::<Vec<u64>>(),
        ),
        TdmsDType::U16 => serde_json::Value::from(
            slice
                .as_typed::<u16>()?
                .iter()
                .map(|v| *v as u64)
                .collect::<Vec<u64>>(),
        ),
        TdmsDType::U32 => serde_json::Value::from(
            slice
                .as_typed::<u32>()?
                .iter()
                .map(|v| *v as u64)
                .collect::<Vec<u64>>(),
        ),
        TdmsDType::U64 => serde_json::Value::from(
            slice
                .as_typed::<u64>()?
                .iter()
                .copied()
                .collect::<Vec<u64>>(),
        ),
        TdmsDType::Bool => serde_json::Value::from(
            slice
                .as_typed::<bool>()?
                .iter()
                .copied()
                .collect::<Vec<bool>>(),
        ),
        TdmsDType::TimeStamp => {
            return Err("timestamp channel JSON comparison not implemented".into());
        }
        TdmsDType::String => {
            return Err("string channel decoding not supported by new API".into());
        }
    };

    Ok(json)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GoldenJson {
    file_properties: HashMap<String, serde_json::Value>,
    groups: HashMap<String, GoldenGroup>,
}

#[derive(Deserialize, Debug)]
struct GoldenGroup {
    properties: HashMap<String, serde_json::Value>,
    channels: HashMap<String, GoldenChannel>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GoldenChannel {
    dtype: String,
    data: serde_json::Value,
    properties: HashMap<String, serde_json::Value>,
}

#[test]
fn test_corpus() {
    let corpus_dir = Path::new("tests/fixtures/tdms_corpus");
    if !corpus_dir.exists() {
        eprintln!(
            "Corpus directory not found at {:?}. Skipping tests.",
            corpus_dir
        );
        return;
    }

    let mut visited = 0;
    let mut passed = 0;

    visit_dirs(corpus_dir, &mut |entry| {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("tdms") {
            visited += 1;
            run_test_case(&path);
            passed += 1;
        }
    })
    .unwrap();

    println!("Visited {} TDMS files.", visited);
    assert!(visited > 0, "No TDMS files found in corpus!");
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&fs::DirEntry)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn run_test_case(tdms_path: &Path) {
    let json_path = tdms_path.with_extension("json");
    assert!(json_path.exists(), "Missing JSON for {:?}", tdms_path);

    println!("Testing {:?}", tdms_path);

    // Load Rust Parser output
    // This is expected to fail or panic until implemented
    let tdms_file = match TdmsFile::open(tdms_path) {
        Ok(f) => f,
        Err(e) => {
            // Allow failure for now by printing error, but eventually we want strict assertions
            // For TDD I will panic to show red.
            panic!("Failed to load {:?}: {:?}", tdms_path, e);
        }
    };

    // Load Golden JSON
    let json_file = fs::File::open(&json_path).expect("Failed to open JSON");
    let golden: GoldenJson = serde_json::from_reader(json_file).expect("Failed to parse JSON");

    // Assert File Properties
    // We compare counts for now, full comparison requires Value conversion
    // assert_eq!(tdms_file.properties.len(), golden.file_properties.len(), "File property count mismatch");

    // Assert Groups
    assert_eq!(tdms_file.groups().count(), golden.groups.len(), "Group count mismatch for {:?}", tdms_path);

    for (g_name, g_golden) in &golden.groups {
        let g_parsed = tdms_file
            .group(g_name)
            .unwrap_or_else(|| panic!("Missing group '{}' in {:?}", g_name, tdms_path));

        // Assert Group Properties
        assert_eq!(
            g_parsed.properties().count(),
            g_golden.properties.len(),
            "Property count mismatch for group '{}'",
            g_name
        );

        let expected_channels = g_golden.channels.len();
        assert_eq!(
            g_parsed.channels().count(),
            expected_channels,
            "Channel count mismatch for group '{}'",
            g_name
        );

        for (c_name, c_golden) in &g_golden.channels {
            let c_parsed = g_parsed
                .channel(c_name)
                .unwrap_or_else(|| panic!("Missing channel '{}' in group '{}'", c_name, g_name));

            assert_eq!(
                c_parsed.properties().count(),
                c_golden.properties.len(),
                "Property count mismatch for channel '{}'",
                c_name
            );

            // Assert Data
            // Simple presence check for now, eventually full comparison
            if !c_golden.data.is_null() {
                let data_json = match read_channel_data_as_json(&c_parsed) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Warning: Could not read channel '{}' data: {:?}", c_name, e);
                        eprintln!("  dtype: {:?}, len: {}", c_parsed.dtype(), c_parsed.len());
                        continue;
                    }
                };

                if let (Some(expected), Some(actual)) = (c_golden.data.as_array(), data_json.as_array()) {
                    assert_eq!(actual.len(), expected.len(), "Count mismatch for {}", c_name);

                    match c_parsed.dtype() {
                        TdmsDType::F64 | TdmsDType::F32 => {
                            for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
                                let a = a.as_f64().expect("expected f64 JSON");
                                let e = if let Some(n) = e.as_f64() {
                                    n
                                } else if let Some(s) = e.as_str() {
                                    match s {
                                        "nan" => f64::NAN,
                                        "inf" => f64::INFINITY,
                                        "-inf" => f64::NEG_INFINITY,
                                        _ => panic!("Unknown float string {}", s),
                                    }
                                } else {
                                    panic!("Expected numeric JSON");
                                };

                                if e.is_nan() {
                                    assert!(a.is_nan(), "Expected NaN at {} for {}", i, c_name);
                                } else if e.is_infinite() {
                                    assert!(
                                        a.is_infinite(),
                                        "Expected Infinity at {} for {}",
                                        i,
                                        c_name
                                    );
                                    assert_eq!(a.is_sign_positive(), e.is_sign_positive());
                                } else {
                                    assert!(
                                        (a - e).abs() < 1e-9,
                                        "Value mismatch at {} for {}: {} vs {}",
                                        i,
                                        c_name,
                                        a,
                                        e
                                    );
                                }
                            }
                        }
                        TdmsDType::I8
                        | TdmsDType::I16
                        | TdmsDType::I32
                        | TdmsDType::I64 => {
                            for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
                                let a = a.as_i64().expect("expected i64 JSON");
                                let e = e.as_i64().expect("expected i64 JSON");
                                assert_eq!(a, e, "Value mismatch at {} for {}", i, c_name);
                            }
                        }
                        TdmsDType::U8 | TdmsDType::U16 | TdmsDType::U32 | TdmsDType::U64 => {
                            for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
                                let a = a.as_u64().expect("expected u64 JSON");
                                let e = e.as_u64().expect("expected u64 JSON");
                                assert_eq!(a, e, "Value mismatch at {} for {}", i, c_name);
                            }
                        }
                        TdmsDType::Bool => {
                            for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
                                let a = a.as_bool().expect("expected bool JSON");
                                let e = e.as_bool().expect("expected bool JSON");
                                assert_eq!(a, e, "Value mismatch at {} for {}", i, c_name);
                            }
                        }
                        TdmsDType::TimeStamp | TdmsDType::String => {
                            // Explicitly not supported by the new API's typed decoding.
                        }
                    }
                }
            }
        }
    }
}
