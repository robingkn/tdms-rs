# tdms-rs

[![Crates.io](https://img.shields.io/crates/v/tdms-rs.svg)](https://crates.io/crates/tdms-rs)
[![Documentation](https://docs.rs/tdms-rs/badge.svg)](https://docs.rs/tdms-rs)
[![License](https://img.shields.io/crates/l/tdms-rs.svg)](#license)

A pure Rust library for reading and writing National Instruments TDMS (Technical Data Management Streaming) files with high performance and zero-copy capabilities.

## 🚀 Key Features

- **⚡ High Performance**: Designed for high-throughput I/O. Achieves near-disk bandwidth by minimizing syscalls and using efficient buffer management.
- **📦 Lazy Loading**: Only loads data when requested. Metadata is indexed eagerly, while raw data is read on-demand to minimize memory footprint.
- **🛡️ Type Safe**: Strongly-typed channel access ensures data integrity at compile time.
- **🔗 Pure Rust**: No external C dependencies, making cross-compilation seamless.
- **📊 Full Format Support**: Supports all TDMS data types, hierarchical structures, and multi-segment files.

## 📖 Documentation

- [**Quick Start Guide**](docs/API.md) - Get up and running in minutes.
- [**Architecture & Design**](docs/ARCHITECTURE.md) - Deep dive into internal reader/writer models and performance tradeoffs.
- [**API Reference** (docs.rs)](https://docs.rs/tdms-rs) - Detailed module and function-level documentation.

## 🛠️ Quick Start

### Reading

```rust
use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::open("data.tdms")?;
    let channel = file.group("Sensors")?.channel("Temperature")?;
    
    let mut data = vec![0.0f64; channel.len()];
    channel.read(0..channel.len(), &mut data)?;
    
    println!("Read {} samples", data.len());
    Ok(())
}
```

### Writing

```rust
use tdms_rs::TdmsWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsWriter::create("output.tdms")?;
    let mut group = writer.add_group("DAQ")?;
    let mut channel = group.add_channel::<f64>("Voltage")?;
    
    channel.write(&[1.0, 2.0, 3.0])?;
    writer.close()?;
    Ok(())
}
```

## 📐 Philosophy & Guarantees

1.  **Memory Efficiency**: Never load data you don't ask for. Metadata is indexed; raw data is lazy-loaded.
2.  **Safety First**: Safe wrappers around `unsafe` memory operations.
3.  **Modern MSRV**: Supports the latest stable Rust features.

## 🤝 License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
