# tdms-rs - Rust TDMS File Library

A pure Rust library for reading and writing National Instruments TDMS (Technical Data Management Streaming) files.

## TDMS Format Overview

### Format Guarantees

This library provides the following guarantees when working with TDMS files:

- **Binary Compatibility**: Files written by this library are fully compatible with National Instruments TDMS readers
- **Deterministic Output**: Channel and group ordering is consistent (alphabetical) across writes
- **Data Integrity**: All TDMS data types are supported with full precision preservation
- **Property Preservation**: File, group, and channel properties are maintained through read/write cycles

### Memory Behavior

The library is designed for efficient memory usage:

- **Zero-Copy Parsing**: Where possible, data is parsed without additional allocations
- **Owned Data**: Channel data is stored in owned vectors for safe access across threads
- **Streaming**: Files are read segment-by-segment to handle large files efficiently
- **Minimal Allocations**: Property parsing and metadata handling minimize memory overhead

### Performance Characteristics

- **Large Files**: The library handles large TDMS files efficiently through streaming
- **Memory Usage**: Peak memory usage is proportional to the largest channel's data size
- **I/O Efficiency**: Sequential reading minimizes disk seeks
- **Write Performance**: Single-segment writes provide optimal write performance

### Deterministic Output

When writing TDMS files:
- Groups are written in alphabetical order by name
- Channels within groups are written in alphabetical order by name  
- Property ordering is deterministic within each object
- Binary output is identical for identical input data

## Features

### Reading TDMS Files
- Zero-copy parsing where possible
- Support for all TDMS data types (integers, floats, strings, timestamps, booleans)
- Comprehensive test coverage with 24 test cases covering edge cases
- No external dependencies beyond standard parsing libraries

### Writing TDMS Files ✨ NEW
- Create TDMS files that match the existing corpus exactly
- Support for all data types: Double, Float, I8-I64, U8-U64, Boolean, String, TimeStamp
- Multi-group, multi-channel file support
- File, group, and channel properties
- Deterministic channel ordering for consistent output
- Corpus-compatible output verified by round-trip testing

### Command-line Tool
- TDMS file validation and structure inspection utility
- Note: Currently validates files but does not convert to JSON

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tdms-rs = "1.0"
```

## Usage

### Writing TDMS Files

#### Minimal Example
```rust
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::TdmsData;

let mut writer = TdmsFileWriter::new("minimal.tdms");
let group = writer.add_group("Group");
group.add_channel("Channel", TdmsData::Double(vec![1.0, 2.0, 3.0]));
writer.write().unwrap();
```

#### Multiple Channels & Data Types
```rust
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::TdmsData;

let mut writer = TdmsFileWriter::new("multi_channel.tdms");
let group = writer.add_group("Sensors");

group.add_channel("Temperature", TdmsData::Double(vec![22.5, 23.0, 24.1]));
group.add_channel("Pressure", TdmsData::I32(vec![101325, 101330, 101320]));
group.add_channel("Valid", TdmsData::Boolean(vec![true, true, false]));
group.add_channel("Labels", TdmsData::String(vec!["A".into(), "B".into(), "C".into()]));

writer.write().unwrap();
```

#### Properties Example
```rust
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::{TdmsData, PropertyValue};

let mut writer = TdmsFileWriter::new("with_properties.tdms");

// File-level properties
writer.add_property("Author", PropertyValue::String("TDMS Writer".into()));
writer.add_property("Version", PropertyValue::I32(1));

let group = writer.add_group("Measurements");
// Group-level properties
group.add_property("Unit_System", PropertyValue::String("SI".into()));
group.add_property("Sample_Rate", PropertyValue::Double(1000.0));

let channel = group.add_channel("Voltage", TdmsData::Double(vec![1.1, 2.2, 3.3]));
// Channel-level properties
channel.add_property("wf_unit_string", PropertyValue::String("V".into()));
channel.add_property("wf_increment", PropertyValue::Double(0.001));

writer.write().unwrap();
```

#### All Supported Data Types
```rust
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::TdmsData;

let mut writer = TdmsFileWriter::new("all_types.tdms");

// Integer types
let integers = writer.add_group("Integers");
integers.add_channel("Int8", TdmsData::I8(vec![-128, 0, 127]));
integers.add_channel("Int16", TdmsData::I16(vec![-32768, 0, 32767]));
integers.add_channel("Int32", TdmsData::I32(vec![-2147483648, 0, 2147483647]));
integers.add_channel("Int64", TdmsData::I64(vec![i64::MIN, 0, i64::MAX]));

// Unsigned integer types
let unsigned = writer.add_group("Unsigned");
unsigned.add_channel("UInt8", TdmsData::U8(vec![0, 128, 255]));
unsigned.add_channel("UInt16", TdmsData::U16(vec![0, 32768, 65535]));
unsigned.add_channel("UInt32", TdmsData::U32(vec![0, 2147483648, 4294967295]));
unsigned.add_channel("UInt64", TdmsData::U64(vec![0, u64::MAX/2, u64::MAX]));

// Floating point types
let floats = writer.add_group("Floats");
floats.add_channel("Float32", TdmsData::Float(vec![1.1, 2.2, 3.3]));
floats.add_channel("Float64", TdmsData::Double(vec![1.1, 2.2, 3.3]));

// Other types
let misc = writer.add_group("Misc");
misc.add_channel("Booleans", TdmsData::Boolean(vec![true, false, true]));
misc.add_channel("Strings", TdmsData::String(vec!["Hello".into(), "World".into()]));
misc.add_channel("Timestamps", TdmsData::TimeStamp(vec![(1000, 0), (2000, 500000000)]));

writer.write().unwrap();
```

### Reading TDMS Files

#### Quick Start

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

#### Reading Channel Data

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

#### Accessing Properties

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

#### Working with Timestamps

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
tdms-to-json input.tdms  # Validates file and shows structure
```

The `tdms-to-json` tool validates TDMS files and displays their structure, including:
- File-level properties
- Group and channel counts
- Data sample counts
- Property summaries

Note: Despite the name, this tool currently validates files but does not convert to JSON.

## Examples

The crate includes several runnable examples demonstrating different use cases:

### Reading Examples
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

### Writing Examples
```bash
# Create a minimal TDMS file
cargo run --example write_minimal

# Create a file with multiple data types
cargo run --example write_multi_channel

# Create a file with properties
cargo run --example write_properties

# Create a comprehensive example with all data types
cargo run --example write_all_types
```

All examples work with the included test corpus if no file path is provided.

## Supported Data Types

| TDMS Type | Rust Type | Description |
|-----------|-----------|-------------|
| I8        | `i8`      | 8-bit signed integer |
| I16       | `i16`     | 16-bit signed integer |
| I32       | `i32`     | 32-bit signed integer |
| I64       | `i64`     | 64-bit signed integer |
| U8        | `u8`      | 8-bit unsigned integer |
| U16       | `u16`     | 16-bit unsigned integer |
| U32       | `u32`     | 32-bit unsigned integer |
| U64       | `u64`     | 64-bit unsigned integer |
| Float     | `f32`     | 32-bit floating point |
| Double    | `f64`     | 64-bit floating point |
| String    | `String`  | UTF-8 encoded text |
| Boolean   | `bool`    | True/false values |
| TimeStamp | `(i64, u64)` | TDMS timestamp (seconds since 1904, fraction) |

Special float values (NaN, Infinity, -0.0) are fully supported.

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
