# `tdms-rs` API Design Documentation

The `tdms-rs` library provides a pure Rust implementation for reading and writing National Instruments TDMS (Technical Data Management Streaming) files. The library design focuses on type safety, hierarchical data access, and performance.

## 1. API Overview

The library functionality is split into two primary domains:
1.  **Reading**: Hierarchical navigation of existing files (`File` → `Group` → `Channel`) with strongly-typed data extraction.
2.  **Writing**: A builder-pattern interface for constructing new files with compile-time type enforcement.

**Key Features**:
*   **Safe**: Safe Rust wrappers around unsafe memory operations (e.g., zero-copy mapping).
*   **Typed**: Channel data is accessed via strongly-typed slices (e.g., `&[f64]`), avoiding `Box<Any>` overhead where possible.
*   **Ergonomic**: Fluent APIs for writing and standard Iterators for reading.

---

## 2. Public Modules

The library exposes a flat public API under `tdms_rs`. Internal modules (such as `reader`, `writer`, `datatypes`) are private, with key types re-exported at the crate root.

| Functionality | Primary Types |
| :--- | :--- |
| **Reading** | `TdmsFile`, `TdmsGroup`, `TdmsChannel`, `TdmsSlice` |
| **Writing** | `TdmsWriter`, `WriterGroup`, `WriterChannel` |
| **Data & Core** | `PropertyValue`, `TdmsData`, `TdmsDType`, `TdmsError` |

---

## 3. Reading API (Methods & Signatures)

These types are used to consume existing TDMS files.

### `TdmsFile`
Represents an indexed TDMS file handle. Safe to share across threads (`Send + Sync`).

```rust
impl TdmsFile {
    /// Open a TDMS file from the filesystem.
    pub fn open(path: impl AsRef<Path>) -> Result<Self>;

    /// Get a group by name. Returns None if not found.
    pub fn group(&self, name: &str) -> Option<TdmsGroup<'_>>;

    /// Iterator over all groups in the file.
    pub fn groups(&self) -> impl Iterator<Item = TdmsGroup<'_>>;

    /// Get a file-level property.
    pub fn property(&self, name: &str) -> Option<&PropertyValue>;

    /// Iterator over all file-level properties.
    pub fn properties(&self) -> impl Iterator<Item = (&str, &PropertyValue)>;
}
```

### `TdmsGroup<'a>`
A handle to a specific group within the file hierarchy.

```rust
impl<'a> TdmsGroup<'a> {
    /// The name of the group.
    pub fn name(&self) -> &str;

    /// Get a specific channel by name.
    pub fn channel(&self, name: &str) -> Option<TdmsChannel<'a>>;

    /// Iterator over all channels in the group.
    pub fn channels(&self) -> impl Iterator<Item = TdmsChannel<'a>>;

    /// Get a group-level property.
    pub fn property(&self, name: &str) -> Option<&PropertyValue>;
}
```

### `TdmsChannel<'a>`
A handle to a specific channel. This is the primary interface for data access.

```rust
impl<'a> TdmsChannel<'a> {
    pub fn name(&self) -> &str;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    /// Get the runtime data type of the channel.
    pub fn dtype(&self) -> TdmsDType;

    /// Read a specific range of data [start..end).
    /// Returns a generic wrapper that can be cast to a typed slice.
    pub fn read(&self, range: Range<usize>) -> Result<TdmsSlice<'a>>;

    /// Read all data in the channel.
    pub fn read_all(&self) -> Result<TdmsSlice<'a>>;

    /// Read directly into a user-provided mutable slice.
    pub fn read_into<T: Pod>(&self, range: Range<usize>, out: &mut [T]) -> Result<usize>;

    /// Iterate over data in chunks (useful for processing large files).
    pub fn chunks(&'a self, chunk_size: usize) -> ChunkIterator<'a>;

    /// Computed iterator for timestamps if standard waveform properties exist.
    pub fn timestamps(&self) -> Option<TimestampIterator>;

    /// Get a channel-level property.
    pub fn property(&self, name: &str) -> Option<&PropertyValue>;
}
```

### `TdmsSlice<'a>`
An abstraction that holds the returned data. It may either own the data (heap allocated) or point to a memory map (zero-copy).

```rust
impl<'a> TdmsSlice<'a> {
    pub fn len(&self) -> usize;
    
    /// Returns true if the slice is backed by a zero-copy mechanism (e.g. mmap).
    pub fn is_zero_copy(&self) -> bool;

    /// View the data as a typed slice (e.g., &'a [f64]).
    /// Returns TypeMismatch if T does not match the channel's actual type.
    pub fn as_typed<T: Pod>(&self) -> Result<&[T]>;
}
```

---

## 4. Writing API (Methods & Signatures)

These types follow a builder pattern to construct valid TDMS files.

### `TdmsWriter`
The top-level writer object.

```rust
impl TdmsWriter {
    /// Create a new TDMS file (overwriting if it exists).
    pub fn create(path: impl AsRef<Path>) -> Result<Self>;

    /// Add a new group (or get a handle to an existing one).
    pub fn add_group(&mut self, name: impl Into<String>) -> Result<WriterGroup<'_>>;

    /// Add a property to the file root.
    pub fn add_property(&mut self, name: impl Into<String>, value: PropertyValue) -> Result<&mut Self>;

    /// Finalize indexes and close the file.
    pub fn close(self) -> Result<()>;
}
```

### `WriterGroup<'w>`
A handle for populating a group.

```rust
impl<'w> WriterGroup<'w> {
    /// Add a channel with a specific static type T.
    /// T must implement WritableType (e.g., f64, i32, String).
    pub fn add_channel<T: WritableType>(&mut self, name: impl Into<String>) -> Result<WriterChannel<'_, T>>;

    /// Add a property to the group.
    pub fn add_property(&mut self, name: impl Into<String>, value: PropertyValue) -> Result<&mut Self>;
}
```

### `WriterChannel<'w, T>`
A typed handle for writing data. The type `T` ensures that only data of the correct type can be written to this channel.

```rust
impl<'w, T: WritableType> WriterChannel<'w, T> {
    /// Write a slice of data to the channel.
    pub fn write(&mut self, data: &[T]) -> Result<()>;

    /// Add a property to the channel.
    pub fn add_property(&mut self, name: impl Into<String>, value: PropertyValue) -> Result<&mut Self>;
}
```

---

## 5. Data Types & Utilities

### `PropertyValue` (Enum)
A rich enum representing all supported TDMS metadata value types.
*   **Numeric**: `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`, `Float` (f32), `Double` (f64)
*   **Other**: `String`, `Boolean`, `TimeStamp` ((i64, u64))

### `TdmsDType` (Enum)
A lightweight runtime enumeration of channel types (`F64`, `I32`, `String`, etc.).

### Traits
*   **`Pod`**: Marker trait for types that can be read safely from bytes (Plain Old Data).
*   **`WritableType`**: Trait for types that can be written to a channel.

---

## 6. Comprehensive Examples

### 6.1 Basic Read Workflow
Opening a file and reading a known channel.

```rust
use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::open("data.tdms")?;
    
    // 1. Access hierarchy
    let group = file.group("Sensors").expect("Group not found");
    let channel = group.channel("Voltage").expect("Channel not found");
    
    // 2. Read data (returns generic slice)
    // The range 0..100 reads the first 100 samples
    let slice = channel.read(0..100)?; 
    
    // 3. Access as typed slice (cheap, checks type compatibility)
    let data: &[f64] = slice.as_typed()?;
    println!("First sample: {:.4}", data[0]);
    
    Ok(())
}
```

### 6.2 Basic Write Workflow
Creating a file with metadata and data.

```rust
use tdms_rs::{TdmsWriter, PropertyValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsWriter::create("output.tdms")?;
    
    let mut group = writer.add_group("Measurement")?;
    
    // Create strongly-typed channel (f64)
    let mut channel = group.add_channel::<f64>("Pressure")?;
    
    // Write data
    channel.write(&[1.0, 1.1, 1.2, 1.3])?;
    channel.add_property("Unit", PropertyValue::String("Bar".into()))?;
    
    writer.close()?;
    Ok(())
}
```

### 6.3 Advanced Reading: Inspecting Unknown Files
Recursively printing the structure of a file.

```rust
use tdms_rs::TdmsFile;

fn inspect_file(path: &str) -> Option<()> {
    let file = TdmsFile::open(path).ok()?;

    println!("File Properties:");
    for (name, value) in file.properties() {
         println!("  {} = {}", name, value);
    }

    for group in file.groups() {
        println!("Group: {}", group.name());
        for (name, value) in group.properties() {
            println!("  Property: {} = {}", name, value);
        }

        for channel in group.channels() {
            println!("  Channel: {} ({} samples, {:?})", 
                channel.name(), channel.len(), channel.dtype());
        }
    }
    Some(())
}
```

### 6.4 Advanced Reading: Chunked Processing
Processing a large channel in blocks to minimize memory usage.

```rust
use tdms_rs::TdmsFile;

fn calculate_average(path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("C").unwrap();
    
    let mut sum = 0.0;
    let mut count = 0;
    
    // Iterate over 1024-sample chunks
    for chunk in channel.chunks(1024) {
        let slice = chunk?;
        let data = slice.as_typed::<f64>()?;
        
        sum += data.iter().sum::<f64>();
        count += data.len();
    }
    
    Ok(if count > 0 { sum / count as f64 } else { 0.0 })
}
```

### 6.5 Advanced Reading: Zero-Allocation with Buffers
Reusing a single buffer to read specific ranges, avoiding repeated allocations.

```rust
use tdms_rs::TdmsFile;

fn read_with_buffer(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::open(path)?;
    let channel = file.group("G").unwrap().channel("C").unwrap();
    
    // Pre-allocate buffer once
    let mut buffer = vec![0.0f64; 100];
    
    // Read directly into buffer
    let samples_read = channel.read_into(0..100, &mut buffer)?;
    
    println!("Read {} samples into buffer: {:?}", samples_read, &buffer[0..samples_read]);
    Ok(())
}
```

### 6.6 Advanced Writing: Generating Synthetic Data
Writing channels dynamically in a loop.

```rust
use tdms_rs::TdmsWriter;
use std::f64::consts::PI;

fn generate_sine_waves() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsWriter::create("sine_waves.tdms")?;
    let mut group = writer.add_group("Generated")?;
    
    let t: Vec<f64> = (0..1000).map(|i| i as f64 * 0.01).collect();
    
    // Create 5 channels: Sine_0, Sine_1, ...
    for i in 0..5 {
        let name = format!("Sine_{}", i);
        let phase = i as f64 * PI / 4.0;
        let data: Vec<f64> = t.iter().map(|&x| (x + phase).sin()).collect();
        
        let mut channel = group.add_channel::<f64>(&name)?;
        channel.write(&data)?;
    }
    
    writer.close()?;
    Ok(())
}
```
