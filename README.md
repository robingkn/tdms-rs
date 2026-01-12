# tdms-rs

[![Crates.io](https://img.shields.io/crates/v/tdms-rs.svg)](https://crates.io/crates/tdms-rs)
[![Documentation](https://docs.rs/tdms-rs/badge.svg)](https://docs.rs/tdms-rs)
[![License](https://img.shields.io/crates/l/tdms-rs.svg)](https://github.com/robingkn/tdms-rs#license)

A pure Rust library for reading and writing National Instruments TDMS (Technical Data Management Streaming) files with full format support and high performance.

## Key Features

- **Complete TDMS Support**: Read and write all TDMS data types and hierarchical structures.
- **High Performance**: Zero-copy parsing where possible and efficient memory usage.
- **Type Safety**: Leverages Rust's type system for safe and ergonomic data handling.
- **Binary Compatibility**: Output files are verified against National Instruments software.
- **Pure Rust**: No external C dependencies, ensuring easy cross-compilation.

## Installation

Add `tdms-rs` to your `Cargo.toml`:

```toml
[dependencies]
tdms-rs = "1.0"
```

## Quick Start

### Reading TDMS Files

```rust
use tdms_rs::TdmsFile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::load(Path::new("data.tdms"))?;
    
    // Direct channel access
    if let Some(channel) = file.get_channel("Sensors", "Temperature") {
        println!("Samples: {}", channel.data_len());
        
        // Type-safe data access
        if let Some(data) = channel.as_f64() {
            let avg = data.iter().sum::<f64>() / data.len() as f64;
            println!("Average: {:.2}", avg);
        }
    }
    
    Ok(())
}
```

### Writing TDMS Files

```rust
use tdms_rs::{TdmsFileWriter, TdmsData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsFileWriter::new("output.tdms");
    
    writer.add_property("Author", "Rust App")?;
    
    let group = writer.add_group("Sensors")?;
    let channel = group.add_channel("Temperature", 
        TdmsData::Double(vec![20.1, 21.5, 22.3]))?;
    
    channel.add_property("wf_unit_string", "°C")?;
    
    writer.write()?;
    Ok(())
}
```

## Supported Data Types

| TDMS Type | Rust Type | Description |
|-----------|-----------|-------------|
| I8-I64    | `i8`-`i64` | Signed integers |
| U8-U64    | `u8`-`u64` | Unsigned integers |
| Float     | `f32`     | 32-bit floating point |
| Double    | `f64`     | 64-bit floating point |
| String    | `String`  | UTF-8 encoded text |
| Boolean   | `bool`    | True/false values |
| TimeStamp | `(i64, u64)`| TDMS timestamp (seconds since 1904, fraction) |

## Command Line Tool

The crate includes `tdms-to-json`, a tool to inspect and validate TDMS files.

```bash
cargo install tdms-rs
tdms-to-json input.tdms
```

## Performance & Guarantees

- **Memory Efficiency**: Streaming reads and minimal allocations during parsing.
- **Data Integrity**: Round-trip tested to ensure data consistency.
- **Deterministic**: Groups and channels are written in consistent order.

## Testing

The library is verified against a corpus of 24+ TDMS scenarios covering multi-segment files, sparse data, and Unicode support.

```bash
cargo test
```

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
