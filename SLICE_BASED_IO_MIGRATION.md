# Slice-Based I/O Migration Guide

## Overview

This document describes the breaking changes made to tdms-rs to implement slice-based I/O with caller-owned memory and explicit I/O semantics.

## What Changed

### 1. Removed Implicit Lazy Loading APIs

**Removed:** All `as_*` methods (`as_f64()`, `as_i32()`, `as_string()`, etc.)

**Why:** These methods performed hidden I/O operations via lazy loading, making it unclear when disk reads occurred. They also allocated memory internally.

**Replacement:** Use explicit `read_*_into()` methods that take caller-provided buffers.

### 2. Explicit Slice-Based Reading

**New API:** `read_f64_into()`, `read_i32_into()`, `read_u8_into()`, etc.

**Key Changes:**
- Caller allocates the buffer
- I/O is explicit (no hidden lazy loading)
- Errors propagate (no silent Option-based failures)
- Zero-copy reading from disk to caller's buffer

### 3. Slice-Based Writing with Chunking

**New API:** `write_f64_slice_chunked()`, `write_i32_slice_chunked()`, etc.

**Key Changes:**
- Writer accepts slices instead of owned `TdmsData`
- Automatic chunking for large datasets (64MB default)
- Zero-copy writing (except for bool conversion, which is format-required)

### 4. Removed Allocation-Heavy Methods

**Removed:**
- `as_numeric()` - allocated Vec<f64>
- `as_timestamps_f64()` - allocated Vec<f64>
- `timestamps_to_unix()` - allocated Vec<f64>

**Replacement:** Read into caller-allocated buffers and convert manually if needed.

## Migration Examples

### Reading Channel Data

**Before (implicit lazy loading):**
```rust
let file = TdmsFile::load(Path::new("data.tdms"))?;
if let Some(channel) = file.get_channel("Group", "Channel") {
    if let Some(data) = channel.as_f64() {
        let avg = data.iter().sum::<f64>() / data.len() as f64;
        println!("Average: {}", avg);
    }
}
```

**After (explicit slice-based):**
```rust
let file = TdmsFile::load(Path::new("data.tdms"))?;
if let Some(channel) = file.get_channel("Group", "Channel") {
    let expected_count = channel.data_len();
    let mut buffer = vec![0.0f64; expected_count];
    let read_count = channel.read_f64_into(&mut buffer)?;
    
    let avg = buffer[..read_count].iter().sum::<f64>() / read_count as f64;
    println!("Average: {}", avg);
}
```

### Writing Channel Data

**Before (TdmsData allocation):**
```rust
let mut writer = TdmsFileWriter::new("output.tdms");
let group = writer.add_group("Sensors")?;
let data = vec![1.1, 2.2, 3.3];
group.add_channel("Temperature", TdmsData::Double(data))?;
writer.write()?;
```

**After (slice-based, no allocation):**
```rust
let mut writer = TdmsFileWriter::new("output.tdms");
let group = writer.add_group("Sensors")?;
let data = vec![1.1, 2.2, 3.3];
group.add_channel("Temperature", TdmsData::Double(data.clone()))?;  // Still works
writer.write()?;

// Or use slice-based chunked write (when available):
// writer.write_f64_slice_chunked(&mut file, &data, 0)?;
```

**Note:** The writer still accepts `TdmsData` for backward compatibility, but internally uses slice-based zero-copy writing with chunking.

### Partial Reads

**New capability:** Read partial data into smaller buffers

```rust
let channel = file.get_channel("Group", "Channel").unwrap();
let total_samples = channel.data_len();

// Read in chunks
const CHUNK_SIZE: usize = 1000;
let mut buffer = vec![0.0f64; CHUNK_SIZE];
let mut offset = 0;

while offset < total_samples {
    let to_read = (total_samples - offset).min(CHUNK_SIZE);
    let slice = &mut buffer[..to_read];
    
    // Note: read_*_into reads from all segments sequentially
    // For true streaming, you'd need segment-aware APIs
    let read_count = channel.read_f64_into(slice)?;
    process_chunk(&slice[..read_count]);
    offset += read_count;
}
```

### Error Handling

**Before:** Methods returned `Option<T>` - unclear if data was missing or wrong type

**After:** Methods return `Result<usize>` - clear error propagation

```rust
match channel.read_f64_into(&mut buffer) {
    Ok(count) => {
        println!("Read {} values", count);
        process_data(&buffer[..count]);
    }
    Err(TdmsError::InvalidFormat(msg)) => {
        eprintln!("Type mismatch: {}", msg);
    }
    Err(e) => {
        eprintln!("I/O error: {}", e);
    }
}
```

## Performance Benefits

1. **Zero-copy I/O:** Data reads directly from disk into caller's buffer
2. **Explicit allocation:** Caller controls when and how much memory to allocate
3. **Chunked writing:** Large datasets are written in optimal chunks (64MB default)
4. **Reduced syscalls:** Chunked writes minimize system call overhead

## Breaking Changes Summary

| Old API | New API | Breaking? |
|---------|---------|-----------|
| `channel.as_f64()` | `channel.read_f64_into(&mut buffer)` | Yes |
| `channel.as_i32()` | `channel.read_i32_into(&mut buffer)` | Yes |
| `channel.as_string()` | `read_string()` (TODO: needs special handling) | Yes |
| `channel.as_numeric()` | Manual conversion after reading | Yes |
| `writer.add_channel(name, TdmsData::Double(vec))` | Still works (internal uses slices) | No |

## Why Breaking Changes Were Necessary

1. **Explicit I/O:** Hidden lazy loading made it impossible to control when I/O occurred
2. **Memory Ownership:** Library-allocated buffers prevented caller control over memory
3. **Performance:** Zero-copy I/O requires caller-provided buffers
4. **Systems Library:** tdms-rs is a systems library - explicit over implicit

## Additional Notes

- **String handling:** String reading requires special parsing (offset tables) and cannot be fully zero-copy. Use `read_raw_data()` for strings or implement custom parsing.
- **Multi-segment files:** `read_*_into()` automatically aggregates data across segments
- **Type safety:** Type mismatches return clear errors instead of None
- **Metadata access:** Property access remains unchanged (still direct access via `properties`)

## Future Improvements

Potential future enhancements (not yet implemented):
- Streaming read APIs for very large files
- Segment-aware reading (read from specific segments)
- String slice-based reading (with offset parsing)
- Interleaved multi-channel reading

