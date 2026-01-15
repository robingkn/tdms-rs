//! Read and process channel data with type matching
//!
//! This example demonstrates how to read numeric channel data,
//! handle different data types, and perform basic analysis.
//!
//! Usage: cargo run --example read_channel_data -- path/to/file.tdms [group_name] [channel_name]

use std::env;
use std::path::Path;
use tdms_rs::{TdmsDType, TdmsFile};

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

    let file = TdmsFile::open(Path::new(file_path))?;
    let group = file
        .group(target_group)
        .ok_or_else(|| format!("Group '{}' not found", target_group))?;
    let channel = group.channel(target_channel).ok_or_else(|| {
        format!(
            "Channel '{}' not found in group '{}'",
            target_channel, target_group
        )
    })?;

    println!("\n✅ Successfully opened channel!");
    println!("dtype = {:?}", channel.dtype());
    println!("len   = {}", channel.len());

    match channel.dtype() {
        TdmsDType::F64 => {
            let slice = channel.read_all()?;
            let values = slice.as_typed::<f64>()?;
            analyze_f64(values);
        }
        TdmsDType::F32 => {
            let slice = channel.read_all()?;
            let values = slice.as_typed::<f32>()?;
            analyze_f32(values);
        }
        TdmsDType::I32 => {
            let slice = channel.read_all()?;
            let values = slice.as_typed::<i32>()?;
            analyze_i32(values);
        }
        TdmsDType::Bool => {
            let slice = channel.read_all()?;
            let values = slice.as_typed::<bool>()?;
            analyze_bool(values);
        }
        TdmsDType::String | TdmsDType::TimeStamp => {
            println!(
                "This example does not decode String/TimeStamp channels in the redesigned API."
            );
        }
        other => {
            println!(
                "This example does not implement analysis for dtype {:?}",
                other
            );
        }
    }

    // Show channel properties if any
    let prop_count = channel.properties().count();
    if prop_count > 0 {
        println!("\n📋 Channel Properties:");
        for (prop_name, prop_value) in channel.properties() {
            println!("  {}: {:?}", prop_name, prop_value);
        }
    }

    Ok(())
}

fn analyze_f64(values: &[f64]) {
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
        print_sample_values(values);
    }
}

fn analyze_f32(values: &[f32]) {
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
        print_sample_values(values);
    }
}

fn analyze_i32(values: &[i32]) {
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
        print_sample_values(values);
    }
}

fn analyze_bool(values: &[bool]) {
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
        print_sample_values(values);
    }
}

fn print_sample_values<T: std::fmt::Display>(values: &[T]) {
    const SAMPLE_SIZE: usize = 5;

    if values.len() <= SAMPLE_SIZE * 2 {
        println!(
            "All values: {:?}",
            values.iter().map(|v| v.to_string()).collect::<Vec<_>>()
        );
    } else {
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
