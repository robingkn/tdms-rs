# Rust Comparison Guide

This guide explains how to use the nptdms benchmark results as a baseline for comparing against the Rust implementation (`tdms-rs`).

## Overview

The benchmark suite is designed to provide fair, reproducible measurements that can be directly compared between Python (`nptdms`) and Rust (`tdms-rs`) implementations. This document explains:

1. How to interpret Python benchmark results
2. How to create equivalent Rust benchmarks
3. What performance differences to expect
4. How to ensure fair comparisons

## Understanding Python Benchmark Results

### Result Schema

All benchmarks output CSV data with this schema:

```csv
benchmark_name,file_type,channels,samples,data_type,operation,time_sec,mb_per_sec,peak_memory_mb,notes
```

**Key Fields:**
- `benchmark_name`: Category of benchmark (read, write, channel_access, stress)
- `file_type`: File size category (small, medium, large, etc.)
- `operation`: Specific operation being measured
- `time_sec`: Wall-clock time in seconds
- `mb_per_sec`: Throughput in MB/s (when applicable)
- `peak_memory_mb`: Peak memory usage during operation

### Python-Specific Overhead

These benchmarks include Python-specific overhead that Rust won't have:

1. **Interpreter Overhead**: Python bytecode execution
2. **GIL Contention**: Global Interpreter Lock (minimal in I/O operations)
3. **Object Creation**: Python object allocation and garbage collection
4. **Type Checking**: Runtime type validation
5. **NumPy Conversion**: Converting to/from NumPy arrays

## Creating Equivalent Rust Benchmarks

### 1. File Operations Mapping

| Python Operation | Rust Equivalent | Notes |
|------------------|-----------------|-------|
| `nptdms.TdmsFile.read()` | `TdmsFile::load()` | File opening and parsing |
| `file['group']['channel']` | `file.get_channel('group', 'channel')` | Channel lookup |
| `channel[:]` | `channel.as_f64()` | Type-safe data access |
| `len(channel)` | `channel.data_len()` | Data length |
| `channel.properties` | `channel.properties` | Property access |

### 2. Benchmark Categories

#### Read Benchmarks (`read_benchmarks.py`)

**Python Operations:**
```python
# File opening
tdms_file = nptdms.TdmsFile.read(path)

# Channel access
channel = tdms_file['group']['channel']
data = channel[:]

# Slicing
slice_data = channel[start:end]
```

**Rust Equivalents:**
```rust
// File opening
let file = TdmsFile::load(path)?;

// Channel access
let channel = file.get_channel("group", "channel")?;
let data = channel.as_f64().unwrap();

// Slicing (if implemented)
let slice_data = &data[start..end];
```

#### Write Benchmarks (`write_benchmarks.py`)

**Python Operations:**
```python
with TdmsWriter(path) as writer:
    channel = ChannelObject('Group', 'Channel', data)
    channel.properties['unit'] = 'V'
    writer.write_data([channel])
```

**Rust Equivalents:**
```rust
let mut writer = TdmsFileWriter::new(path);
let group = writer.add_group("Group")?;
let channel = group.add_channel("Channel", TdmsData::Double(data))?;
channel.add_property("unit", "V")?;
writer.write()?;
```

#### Channel Access Benchmarks (`channel_access_benchmarks.py`)

**Python Operations:**
```python
# Lookup patterns
channel = file['group']['channel']  # Direct indexing
for group in file.groups():         # Iteration
    for channel in group.channels():
        pass

# Data access
full_data = channel[:]              # Full access
chunk = channel[i:i+1000]          # Chunked access
element = channel[i]                # Single element

# Property access
value = channel.properties['key']   # Property lookup
for key, val in channel.properties.items():  # Iteration
    pass
```

**Rust Equivalents:**
```rust
// Lookup patterns
let channel = file.get_channel("group", "channel")?;  // Direct lookup
for (group_name, group) in &file.groups {             // Iteration
    for (channel_name, channel) in &group.channels {
        // Process channel
    }
}

// Data access
let full_data = channel.as_f64().unwrap();            // Full access
let chunk = &full_data[i..i+1000];                    // Chunked access
let element = full_data[i];                           // Single element

// Property access
let value = channel.properties.get("key");            // Property lookup
for (key, value) in &channel.properties {             // Iteration
    // Process property
}
```

### 3. Test File Compatibility

The benchmark suite generates TDMS files that can be used by both implementations:

**Generated Files:**
- `small_*.tdms`: 1-10 MB files for basic testing
- `medium_*.tdms`: 100-500 MB files for throughput testing
- `large_*.tdms`: 1-5 GB files for stress testing
- `stress_*.tdms`: Pathological cases (many channels, properties, etc.)

**Reusing Files in Rust:**
```rust
// Use the same test files
let test_file = "benchmarks/test_files/small_single_channel.tdms";
let file = TdmsFile::load(test_file)?;
```

## Expected Performance Differences

### Where Rust Should Be Faster

1. **File Parsing**: 2-10x faster due to zero-copy parsing
2. **Memory Usage**: 50-80% less memory due to efficient data structures
3. **Data Access**: 5-20x faster for numeric operations
4. **Property Access**: 2-5x faster due to direct memory access

### Where Performance Might Be Similar

1. **I/O Operations**: Limited by disk/network speed
2. **Large File Operations**: I/O bound rather than CPU bound

### Python Advantages

1. **NumPy Integration**: Highly optimized for numerical computing
2. **Ecosystem**: Rich scientific computing libraries
3. **Flexibility**: Dynamic typing and runtime introspection

## Fair Comparison Guidelines

### 1. Use Identical Test Data

```bash
# Generate test files once
cd benchmarks
python generate_test_files.py full

# Use same files in both implementations
python read_benchmarks.py          # Python benchmarks
cargo run --release --bin rust_benchmarks  # Rust benchmarks
```

### 2. Measure Equivalent Operations

**✅ Fair Comparisons:**
- File opening time
- Full channel data access
- Property enumeration
- Memory usage for same operations

**❌ Unfair Comparisons:**
- Python NumPy operations vs Rust raw data
- Python object creation overhead
- Different algorithms or data structures

### 3. Account for Implementation Differences

**Python Characteristics:**
- Interpreted language overhead
- Garbage collection pauses
- Dynamic type checking
- Rich object model

**Rust Characteristics:**
- Compiled native code
- Zero-cost abstractions
- Static type checking
- Manual memory management

### 4. Use Appropriate Build Modes

**Python:**
```bash
# Use standard Python (no special optimizations)
python benchmarks.py
```

**Rust:**
```bash
# Use release mode for fair comparison
cargo run --release --bin benchmarks
```

## Benchmark Implementation Template

Here's a template for creating equivalent Rust benchmarks:

```rust
use std::time::Instant;
use tdms_rs::TdmsFile;

fn benchmark_file_opening(test_files: &[&str]) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    for file_path in test_files {
        let start = Instant::now();
        
        // Equivalent to: nptdms.TdmsFile.read(file_path)
        let file = TdmsFile::load(file_path).unwrap();
        
        let elapsed = start.elapsed();
        
        // Count channels (equivalent to Python counting)
        let total_channels: usize = file.groups.values()
            .map(|group| group.channels.len())
            .sum();
        
        results.push(BenchmarkResult {
            benchmark_name: "read_file_open".to_string(),
            file_type: extract_file_type(file_path),
            channels: total_channels,
            samples: 0, // Metadata only
            data_type: "metadata".to_string(),
            operation: "open_file".to_string(),
            time_sec: elapsed.as_secs_f64(),
            mb_per_sec: 0.0, // Not applicable
            peak_memory_mb: 0.0, // Would need memory profiling
            notes: "File opening and metadata parsing only".to_string(),
        });
    }
    
    results
}

fn benchmark_channel_access(test_files: &[&str]) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    for file_path in test_files {
        let file = TdmsFile::load(file_path).unwrap();
        
        // Find first available channel
        if let Some((group_name, group)) = file.groups.iter().next() {
            if let Some((channel_name, channel)) = group.channels.iter().next() {
                let start = Instant::now();
                
                // Equivalent to: channel[:]
                if let Some(data) = channel.as_f64() {
                    let _len = data.len(); // Force evaluation
                }
                
                let elapsed = start.elapsed();
                
                results.push(BenchmarkResult {
                    benchmark_name: "read_single_channel".to_string(),
                    // ... fill in other fields
                    time_sec: elapsed.as_secs_f64(),
                    // ...
                });
            }
        }
    }
    
    results
}
```

## Interpreting Results

### Performance Metrics

1. **Absolute Time**: Direct comparison of operation duration
2. **Throughput**: MB/s for I/O intensive operations
3. **Memory Usage**: Peak memory consumption
4. **Scalability**: Performance with increasing data size

### Expected Rust Improvements

Based on typical Rust vs Python performance characteristics:

| Operation | Expected Rust Speedup | Confidence |
|-----------|----------------------|------------|
| File parsing | 3-8x | High |
| Data access | 5-15x | High |
| Property access | 2-5x | Medium |
| Memory usage | 50-70% reduction | High |
| Large file I/O | 1.2-2x | Medium |

### Red Flags

**If Rust is slower than Python:**
- Check for debug builds (use `--release`)
- Look for unnecessary allocations
- Verify equivalent operations
- Check for algorithmic differences

**If improvements are too good:**
- Verify measurements are accurate
- Check for compiler optimizations eliminating work
- Ensure equivalent functionality

## Reporting Results

### Comparison Format

```markdown
# nptdms vs tdms-rs Performance Comparison

## Test Environment
- Python: 3.11.0 with nptdms 1.3.0
- Rust: 1.70.0 with tdms-rs 1.0.0
- Hardware: [specify]
- OS: [specify]

## Results Summary

| Operation | Python (s) | Rust (s) | Speedup | Memory Reduction |
|-----------|------------|----------|---------|------------------|
| File Opening | 0.125 | 0.032 | 3.9x | 45% |
| Channel Read | 0.089 | 0.012 | 7.4x | 62% |
| Property Access | 0.045 | 0.018 | 2.5x | 38% |

## Detailed Analysis
[Include detailed breakdown by file type, operation, etc.]
```

### Validation Checklist

- [ ] Same test files used for both implementations
- [ ] Equivalent operations measured
- [ ] Release builds used for Rust
- [ ] Multiple runs averaged
- [ ] Memory measurements included
- [ ] Edge cases tested
- [ ] Results are reproducible

## Conclusion

This benchmark suite provides a solid foundation for fair comparison between nptdms and tdms-rs. The key to meaningful results is ensuring equivalent operations are measured while accounting for the fundamental differences between Python and Rust implementations.

The benchmarks are designed to be honest about Python's limitations while providing clear targets for Rust optimization. Use them as a baseline to demonstrate the performance benefits of the Rust implementation while maintaining compatibility and correctness.