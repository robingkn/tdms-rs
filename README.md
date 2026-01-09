# tdms-rs - Rust TDMS File Parser

A pure Rust library for reading National Instruments TDMS (Technical Data Management Streaming) files.

## Features
- Zero-copy parsing where possible
- Support for all TDMS data types (integers, floats, strings, timestamps, booleans)
- Comprehensive test coverage with 24 test cases covering edge cases
- No external dependencies beyond standard parsing libraries
- Command-line tool for TDMS to JSON conversion

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tdms-rs = "0.1"
```

## Usage

### Quick Start

```rust
use tdms_rs::TdmsFile;
use std::path::Path;

// Load a TDMS file
let file = TdmsFile::load(Path::new("data.tdms"))?;

// Iterate through groups and channels
for (group_name, group) in &file.groups {
    println!("Group: {}", group_name);
    for (channel_name, channel) in &group.channels {
        if let Some(data) = &channel.data {
            println!("  Channel {}: {} samples", channel_name, data.len());
        }
    }
}
```

### Reading Channel Data

```rust
use tdms_rs::{TdmsFile, TdmsData};
use std::path::Path;

let file = TdmsFile::load(Path::new("measurements.tdms"))?;

if let Some(group) = file.groups.get("Sensors") {
    if let Some(channel) = group.channels.get("Temperature") {
        match &channel.data {
            Some(TdmsData::Double(values)) => {
                println!("Temperature readings: {:?}", values);
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                println!("Average temperature: {:.2}°C", avg);
            },
            Some(TdmsData::Float(values)) => {
                println!("Temperature readings: {:?}", values);
            },
            Some(other) => println!("Unexpected data type: {:?}", other),
            None => println!("No data in channel"),
        }
    }
}
```

### Accessing Properties

```rust
use tdms_rs::{TdmsFile, PropertyValue};
use std::path::Path;

let file = TdmsFile::load(Path::new("data.tdms"))?;

// Access group properties
if let Some(group) = file.groups.get("DAQmx") {
    for (prop_name, prop_value) in &group.properties {
        match prop_value {
            PropertyValue::String(s) => println!("Property {}: {}", prop_name, s),
            PropertyValue::Double(d) => println!("Property {}: {}", prop_name, d),
            PropertyValue::I32(i) => println!("Property {}: {}", prop_name, i),
            _ => println!("Property {}: {:?}", prop_name, prop_value),
        }
    }
    
    // Access channel properties
    if let Some(channel) = group.channels.get("Voltage") {
        if let Some(PropertyValue::String(unit)) = channel.properties.get("wf_unit_string") {
            println!("Channel unit: {}", unit);
        }
    }
}
```

### Working with Timestamps

```rust
use tdms_rs::{TdmsFile, TdmsData};
use std::path::Path;

let file = TdmsFile::load(Path::new("time_series.tdms"))?;

if let Some(group) = file.groups.get("Time Data") {
    if let Some(channel) = group.channels.get("Events") {
        if let Some(TdmsData::TimeStamp(timestamps)) = &channel.data {
            for (seconds, fraction) in timestamps {
                // TDMS timestamps are seconds since 1904-01-01 00:00:00 UTC
                // with 2^-64 second precision in the fraction
                println!("Event at: {} seconds + {} fraction", seconds, fraction);
            }
        }
    }
}
```

### Binary Tool
```bash
cargo install tdms-rs
tdms-to-json input.tdms output.json
```

## Examples

The crate includes several runnable examples demonstrating different use cases:

```bash
# Basic file structure inspection
cargo run --example read_file -- path/to/file.tdms

# Detailed channel and property listing
cargo run --example list_channels -- path/to/file.tdms

# Read and analyze specific channel data
cargo run --example read_channel_data -- path/to/file.tdms [group] [channel]

# Display all properties with explanations
cargo run --example read_properties -- path/to/file.tdms
```

All examples work with the included test corpus if no file path is provided.

## Supported Data Types
- Integers: I8, I16, I32, I64, U8, U16, U32, U64
- Floating point: Float (f32), Double (f64)
- Strings with UTF-8 encoding
- Timestamps with high precision
- Booleans
- Special float values (NaN, Infinity, -0.0)

## Testing
The library includes a comprehensive test corpus with 24 different TDMS files covering:
- Basic file structures
- All data types
- Edge cases and numeric limits
- Unicode support
- Large and sparse data
- Multiple segments and incremental writes

```bash
cargo test
```

## Development
Python development tools are available in the `tools/` directory for corpus generation and debugging. See `tools/README.md` for details.
