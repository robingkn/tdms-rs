//! Ergonomic TDMS file reading example
//!
//! This example demonstrates improved ways to access TDMS data using
//! convenience methods and ergonomic patterns. It shows both current
//! and improved approaches for comparison.

use std::env;
use std::path::Path;
use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "tests/fixtures/tdms_corpus/06_properties/all_levels.tdms"
    };

    println!("🔍 Demonstrating ergonomic TDMS reading patterns");
    println!("File: {}", file_path);

    let file = TdmsFile::open(Path::new(file_path))?;

    println!("\n📋 Iteration Patterns:");

    for group in file.groups() {
        println!("   Group: {}", group.name());
        for (k, v) in group.properties() {
            println!("     Group property {}: {:?}", k, v);
        }

        for channel in group.channels() {
            println!("     Channel: {}", channel.name());
            println!("       dtype = {:?}", channel.dtype());
            println!("       len   = {}", channel.len());

            // Demonstrate reading a slice without including allocations in hot loops.
            // This example reads the whole channel once for display.
            if channel.dtype() == tdms_rs::TdmsDType::F64 {
                let slice = channel.read_all()?;
                let values = slice.as_typed::<f64>()?;
                if let Some(first) = values.first() {
                    println!("       first = {:.6}", first);
                }
            }

            for (k, v) in channel.properties() {
                println!("       Channel property {}: {:?}", k, v);
            }
        }
    }

    println!("\n✨ Ergonomic reading demonstration complete!");

    Ok(())
}
