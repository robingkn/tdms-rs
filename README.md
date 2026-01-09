# tdms-rs

[![Crates.io](https://img.shields.io/crates/v/tdms-rs.svg)](https://crates.io/crates/tdms-rs)
[![Documentation](https://docs.rs/tdms-rs/badge.svg)](https://docs.rs/tdms-rs)
[![License](https://img.shields.io/crates/l/tdms-rs.svg)](https://github.com/robingkn/tdms-rs#license)

A pure Rust library for reading and writing National Instruments TDMS (Technical Data Management Streaming) files with full format support and excellent performance.

## Features

- **Complete TDMS Support**: Read and write all TDMS data types and structures
- **High Performance**: Zero-copy parsing and efficient memory usage
- **Type Safety**: Rust's type system prevents common data handling errors
- **Production Ready**: Comprehensive test coverage with 24+ test scenarios
- **Binary Compatibility**: Output files work with National Instruments software
- **Pure Rust**: Minimal external dependencies

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
tdms-rs = "1.0"
```

### Reading TDMS Files

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
            
            // Access typed data
            match data {
                tdms_rs::TdmsData::Double(values) => {
                    let avg = values.iter().sum::<f64>() / values.len() as f64;
                    println!("    Average: {:.2}", avg);
                },
                tdms_rs::TdmsData::I32(values) => {
                    println!("    Range: {} to {}", 
                        values.iter().min().unwrap(),
                        values.iter().max().unwrap());
                },
                _ => println!("    Other data type"),
            }
        }
        
        // Access properties
        for (prop_name, prop_value) in &channel.properties {
            println!("    Property {}: {}", prop_name, prop_value);
        }
    }
}
```

### Writing TDMS Files

```rust
use tdms_rs::{TdmsFileWriter, TdmsData, PropertyValue};

// Create a new TDMS file
let mut writer = TdmsFileWriter::new("output.tdms");

// Add file-level properties
writer.add_property("Author", PropertyValue::String("Rust App".into()));
writer.add_property("Version", PropertyValue::I32(1));

// Create a group with channels
let group = writer.add_group("Sensors");
group.add_property("Location", PropertyValue::String("Lab A".into()));

// Add channels with different data types
group.add_channel("Temperature", TdmsData::Double(vec![20.1, 21.5, 22.3]));
group.add_channel("Pressure", TdmsData::I32(vec![1013, 1015, 1012]));
group.add_channel("Valid", TdmsData::Boolean(vec![true, true, false]));

// Add channel properties
let voltage_channel = group.add_channel("Voltage", TdmsData::Double(vec![1.1, 2.2, 3.3]));
voltage_channel.add_property("wf_unit_string", PropertyValue::String("V".into()));
voltage_channel.add_property("wf_increment", PropertyValue::Double(0.001));

// Write the file
writer.write()?;
```

## API Overview

### Core Types

- **`TdmsFile`**: Root container with groups and file-level properties
- **`TdmsGroup`**: Container for related channels with group-level properties  
- **`TdmsChannel`**: Individual data stream with properties and typed data
- **`TdmsData`**: Enum containing all supported TDMS data types
- **`PropertyValue`**: Enum for metadata values (strings, numbers, timestamps, etc.)

### Reading API

```rust
use tdms_rs::{TdmsFile, TdmsData, PropertyValue};
use std::path::Path;

let file = TdmsFile::load(Path::new("data.tdms"))?;

// Access file properties
for (name, value) in &file.properties {
    match value {
        PropertyValue::String(s) => println!("File property {}: {}", name, s),
        PropertyValue::Double(d) => println!("File property {}: {}", name, d),
        PropertyValue::I32(i) => println!("File property {}: {}", name, i),
        _ => println!("File property {}: {:?}", name, value),
    }
}

// Access groups and channels
if let Some(group) = file.groups.get("Sensors") {
    if let Some(channel) = group.channels.get("Temperature") {
        match &channel.data {
            Some(TdmsData::Double(values)) => {
                println!("Temperature data: {:?}", values);
            },
            Some(TdmsData::I32(values)) => {
                println!("Integer data: {:?}", values);
            },
            Some(TdmsData::String(values)) => {
                println!("String data: {:?}", values);
            },
            Some(TdmsData::TimeStamp(values)) => {
                for (seconds, fraction) in values {
                    println!("Timestamp: {} seconds + {} fraction", seconds, fraction);
                }
            },
            _ => println!("Other or no data"),
        }
    }
}
```

### Writing API

```rust
use tdms_rs::{TdmsFileWriter, TdmsData, PropertyValue};

let mut writer = TdmsFileWriter::new("output.tdms");

// File properties
writer.add_property("Title", PropertyValue::String("Test Data".into()));

// Create groups and channels
let group = writer.add_group("Data");
group.add_property("Description", PropertyValue::String("Sensor readings".into()));

// Add channels with data
group.add_channel("Channel1", TdmsData::Double(vec![1.0, 2.0, 3.0]));
group.add_channel("Channel2", TdmsData::I32(vec![10, 20, 30]));

// Add channel properties
let channel = group.add_channel("Channel3", TdmsData::String(vec!["A".into(), "B".into()]));
channel.add_property("Description", PropertyValue::String("Labels".into()));

writer.write()?;
```

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

All data types support special values (NaN, Infinity) and edge cases.

## Examples

The repository includes comprehensive examples:

### Reading Examples
```bash
cargo run --example read_file -- data.tdms
cargo run --example list_channels -- data.tdms  
cargo run --example read_channel_data -- data.tdms Group Channel
cargo run --example read_properties -- data.tdms
```

### Writing Examples  
```bash
cargo run --example write_minimal
cargo run --example write_multi_channel
cargo run --example write_properties
cargo run --example write_all_types
```

## Performance & Guarantees

### Memory Efficiency
- **Zero-copy parsing** where possible
- **Streaming reads** for large files
- **Minimal allocations** during parsing
- **Owned data** for safe multi-threading

### Binary Compatibility
- **Corpus verified**: Output matches National Instruments reference files
- **Round-trip tested**: Write → read cycles preserve all data
- **Deterministic output**: Consistent file generation

### Format Guarantees
- **Data Integrity**: All TDMS data types supported with full precision
- **Property Preservation**: File, group, and channel metadata maintained
- **Deterministic Output**: Groups and channels written in alphabetical order
- **Binary Compatibility**: Files work with National Instruments software

## Command Line Tool

Install the binary tool:

```bash
cargo install tdms-rs
```

Validate and inspect TDMS files:

```bash
tdms-to-json input.tdms
```

The tool displays file structure, validates format, and shows property summaries.

## Error Handling

The library uses Rust's `Result` type for comprehensive error handling:

```rust
use tdms_rs::TdmsFile;
use std::path::Path;

match TdmsFile::load(Path::new("data.tdms")) {
    Ok(file) => {
        println!("Successfully loaded {} groups", file.groups.len());
        // Process file...
    },
    Err(e) => {
        eprintln!("Failed to load TDMS file: {}", e);
        // Handle error...
    }
}
```

Common error scenarios:
- File not found or permission denied
- Invalid TDMS file format
- Corrupted or truncated files
- Unsupported TDMS features

## Testing

The library includes comprehensive test coverage:

```bash
cargo test
```

Test corpus includes 24+ TDMS files covering:
- Basic file structures
- All data types and edge cases
- Unicode support
- Large and sparse data
- Multi-segment files
- Property metadata

## Version 1.0.0 Release

This is the first stable release of tdms-rs, providing:

- **Production Ready**: Complete TDMS read/write support with comprehensive testing
- **All Data Types**: Full support for TDMS data types and properties
- **Writer API**: Create TDMS files from Rust data structures
- **Binary Compatibility**: Output verified against National Instruments corpus
- **Comprehensive Testing**: 24+ test scenarios covering edge cases and data types
- **Semantic Versioning Promise**: Breaking changes only in major versions

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass with `cargo test`
5. Follow Rust formatting with `cargo fmt`
6. Submit a pull request

### Development Setup

```bash
git clone https://github.com/robingkn/tdms-rs.git
cd tdms-rs
cargo test
cargo run --example write_minimal
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- National Instruments for the TDMS format specification
- The Rust community for excellent tooling and libraries
- Contributors who helped improve the library
