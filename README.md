# TDMS - Rust TDMS File Parser

A pure Rust library for reading National Instruments TDMS (Technical Data Management Streaming) files.

## Features
- Zero-copy parsing where possible
- Support for all TDMS data types (integers, floats, strings, timestamps, booleans)
- Comprehensive test coverage with 24 test cases covering edge cases
- No external dependencies beyond standard parsing libraries
- Command-line tool for TDMS to JSON conversion

## Usage

### Library
```rust
use tdms::TdmsFile;

let file = TdmsFile::load("data.tdms")?;
for (group_name, group) in &file.groups {
    for (channel_name, channel) in &group.channels {
        if let Some(data) = &channel.data {
            println!("Channel {}/{}: {} samples", group_name, channel_name, data.len());
        }
    }
}
```

### Binary Tool
```bash
cargo install tdms
tdms_to_json input.tdms output.json
```

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
