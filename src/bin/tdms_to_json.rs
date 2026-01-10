use std::env;
use std::path::PathBuf;
use tdms_rs::{TdmsData, TdmsFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tdms-to-json <file.tdms>");
        eprintln!("       Validates TDMS file format and reports structure");
        eprintln!("       Note: This tool validates files but does not convert to JSON");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    println!("Validating TDMS file: {:?}", path);

    match TdmsFile::load(&path) {
        Ok(file) => {
            println!("✓ File loaded successfully");
            println!("  File properties: {}", file.properties.len());
            println!("  Groups: {}", file.groups.len());

            for (group_name, group) in &file.groups {
                println!(
                    "  Group '{}': {} channels, {} properties",
                    group_name,
                    group.channels.len(),
                    group.properties.len()
                );

                for (channel_name, channel) in &group.channels {
                    let data_info = match &channel.data {
                        Some(TdmsData::Double(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::Float(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::I8(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::I16(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::I32(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::I64(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::U8(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::U16(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::U32(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::U64(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::Boolean(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::String(v)) => format!("{} samples", v.len()),
                        Some(TdmsData::TimeStamp(v)) => format!("{} samples", v.len()),
                        None => "no data".to_string(),
                    };
                    println!(
                        "    Channel '{}': {}, {} properties",
                        channel_name,
                        data_info,
                        channel.properties.len()
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load file: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
