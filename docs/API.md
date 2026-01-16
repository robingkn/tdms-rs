# tdms-rs API Overview

This document provides a high-level overview of the `tdms-rs` public API. For detailed function documentation, please refer to the [rustdoc](https://docs.rs/tdms-rs).

## Core Concepts

### Reading Files
The hierarchy follows the TDMS standard: `TdmsFile` → `TdmsGroup` → `TdmsChannel`.

```rust
let file = TdmsFile::open("data.tdms")?;
let group = file.group("Sensors").ok_or("Group not found")?;
let channel = group.channel("Temperature").ok_or("Channel not found")?;

// Read into a pre-allocated buffer
let mut data = vec![0.0f64; channel.len()];
channel.read_into(0..channel.len(), &mut data)?;
```

### Writing Files
Writing uses a builder-like pattern through `TdmsWriter`.

```rust
let mut writer = TdmsWriter::create("output.tdms")?;
let mut group = writer.add_group("Measurement")?;
let mut channel = group.add_channel::<f64>("Voltage")?;

channel.write(&[1.0, 2.0, 3.0])?;
writer.close()?;
```

## Data Types
The following types are supported for properties and channel data:
- **Integers**: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`
- **Floats**: `f32`, `f64`
- **Strings**: `String`
- **Boolean**: `bool`
- **Time**: `TimeStamp` (custom struct)

## Advanced Features

### Chunked Reading
To process large files without loading everything into memory, you can iterate over ranges:
```rust
let chunk_size = 1024;
let total_len = channel.len();
let mut buffer = vec![0.0f64; chunk_size];

for start in (0..total_len).step_by(chunk_size) {
    let end = (start + chunk_size).min(total_len);
    let count = end - start;
    channel.read_into(start..end, &mut buffer[0..count])?;
    // process buffer[0..count]...
}
```

### Direct Buffer Access
Minimize allocations by reading into a pre-allocated buffer:
```rust
let mut buffer = vec![0.0f64; 1000];
channel.read_into(0..1000, &mut buffer)?;
```
