//! Read and process channel data with type matching
//!
//! This example demonstrates how to read numeric channel data,
//! handle different data types, and perform basic analysis.
//!
//! Usage: cargo run --example read_channel_data -- path/to/file.tdms [group_name] [channel_name]

use std::env;
use std::path::Path;
use tdms_rs::{TdmsData, TdmsFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let file_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("tests/fixtures/tdms_corpus/03_datatypes/floats.tdms");
    let target_group = args.get(2).map(|s| s.as_str()).unwrap_or("Floats");
    let target_channel = args.get(3).map(|s| s.as_str()).unwrap_or("Float64");

    println!("📊 Reading channel data from: {}", file_path);
    println!(
        "Target: Group '{}', Channel '{}'",
        target_group, target_channel
    );

    let file = TdmsFile::load(Path::new(file_path))?;

    // Find the specified group
    let group = file
        .groups
        .get(target_group)
        .ok_or_else(|| format!("Group '{}' not found", target_group))?;

    // Find the specified channel
    let channel = group.channels.get(target_channel).ok_or_else(|| {
        format!(
            "Channel '{}' not found in group '{}'",
            target_channel, target_group
        )
    })?;

    // Process the channel data
    match &channel.data {
        Some(data) => {
            println!("\n✅ Found channel data!");
            analyze_data(data, target_channel);
        }
        None => {
            println!("❌ No data found in channel '{}'", target_channel);
            return Ok(());
        }
    }

    // Show channel properties if any
    if !channel.properties.is_empty() {
        println!("\n📋 Channel Properties:");
        for (prop_name, prop_value) in &channel.properties {
            println!("  {}: {:?}", prop_name, prop_value);
        }
    }

    Ok(())
}

fn analyze_data(data: &TdmsData, _channel_name: &str) {
    match data {
        TdmsData::Double(values) => {
            println!("Data type: f64 (double precision)");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let sum: f64 = values.iter().sum();
                let mean = sum / values.len() as f64;

                println!("Statistics:");
                println!("  Min: {:.6}", min);
                println!("  Max: {:.6}", max);
                println!("  Mean: {:.6}", mean);

                // Show first and last few values
                print_sample_values(values, "f64");

                // Check for special values
                let nan_count = values.iter().filter(|&&x| x.is_nan()).count();
                let inf_count = values.iter().filter(|&&x| x.is_infinite()).count();
                if nan_count > 0 || inf_count > 0 {
                    println!("Special values: {} NaN, {} Infinite", nan_count, inf_count);
                }
            }
        }

        TdmsData::Float(values) => {
            println!("Data type: f32 (single precision)");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                let min = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                let max = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                let sum: f32 = values.iter().sum();
                let mean = sum / values.len() as f32;

                println!("Statistics:");
                println!("  Min: {:.3}", min);
                println!("  Max: {:.3}", max);
                println!("  Mean: {:.3}", mean);

                print_sample_values(values, "f32");
            }
        }

        TdmsData::I32(values) => {
            println!("Data type: i32 (32-bit signed integer)");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                let min = *values.iter().min().unwrap();
                let max = *values.iter().max().unwrap();
                let sum: i64 = values.iter().map(|&x| x as i64).sum();
                let mean = sum as f64 / values.len() as f64;

                println!("Statistics:");
                println!("  Min: {}", min);
                println!("  Max: {}", max);
                println!("  Mean: {:.2}", mean);

                print_sample_values(values, "i32");
            }
        }

        TdmsData::I64(values) => {
            println!("Data type: i64 (64-bit signed integer)");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                let min = *values.iter().min().unwrap();
                let max = *values.iter().max().unwrap();

                println!("Statistics:");
                println!("  Min: {}", min);
                println!("  Max: {}", max);

                print_sample_values(values, "i64");
            }
        }

        TdmsData::String(values) => {
            println!("Data type: String");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                let lengths: Vec<usize> = values.iter().map(|s| s.len()).collect();
                let min_len = *lengths.iter().min().unwrap();
                let max_len = *lengths.iter().max().unwrap();
                let avg_len = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;

                println!("String length statistics:");
                println!("  Min length: {}", min_len);
                println!("  Max length: {}", max_len);
                println!("  Average length: {:.1}", avg_len);

                println!("Sample strings:");
                for (i, s) in values.iter().take(5).enumerate() {
                    println!("  [{}]: \"{}\"", i, s);
                }
                if values.len() > 5 {
                    println!("  ... and {} more", values.len() - 5);
                }
            }
        }

        TdmsData::Boolean(values) => {
            println!("Data type: bool");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                let true_count = values.iter().filter(|&&x| x).count();
                let false_count = values.len() - true_count;

                println!("Boolean distribution:");
                println!(
                    "  True: {} ({:.1}%)",
                    true_count,
                    100.0 * true_count as f64 / values.len() as f64
                );
                println!(
                    "  False: {} ({:.1}%)",
                    false_count,
                    100.0 * false_count as f64 / values.len() as f64
                );

                print_sample_values(values, "bool");
            }
        }

        TdmsData::TimeStamp(values) => {
            println!("Data type: TimeStamp (TDMS timestamp)");
            println!("Sample count: {}", values.len());

            if !values.is_empty() {
                println!("Note: Timestamps are seconds since 1904-01-01 00:00:00 UTC");
                println!("Format: (seconds, fraction) where fraction is in 2^-64 units");

                println!("Sample timestamps:");
                for (i, (seconds, fraction)) in values.iter().take(5).enumerate() {
                    println!("  [{}]: {} seconds + {} fraction", i, seconds, fraction);
                }
                if values.len() > 5 {
                    println!("  ... and {} more", values.len() - 5);
                }
            }
        }

        _ => {
            println!("Data type: {:?}", data);
            println!("This data type is not handled by this example.");
        }
    }
}

fn print_sample_values<T: std::fmt::Display>(values: &[T], _type_name: &str) {
    const SAMPLE_SIZE: usize = 5;

    if values.len() <= SAMPLE_SIZE * 2 {
        // Show all values if there aren't many
        println!(
            "All values: {:?}",
            values.iter().map(|v| v.to_string()).collect::<Vec<_>>()
        );
    } else {
        // Show first and last few values
        let first: Vec<String> = values
            .iter()
            .take(SAMPLE_SIZE)
            .map(|v| v.to_string())
            .collect();
        let last: Vec<String> = values
            .iter()
            .rev()
            .take(SAMPLE_SIZE)
            .rev()
            .map(|v| v.to_string())
            .collect();

        println!("First {} values: [{}]", SAMPLE_SIZE, first.join(", "));
        println!("Last {} values: [{}]", SAMPLE_SIZE, last.join(", "));
    }
}
