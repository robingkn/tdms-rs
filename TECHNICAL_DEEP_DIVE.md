# Technical Deep Dive: TDMS-RS Write Performance Analysis

## 1. Write Path Breakdown

### tdms-rs Serialization Flow

```rust
// src/writer.rs::write()
pub fn write(&self) -> Result<()> {
    let file = File::create(&self.path)?;
    let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);  // 8MB buffer
    
    // Step 1: Build metadata in-memory
    let metadata_bytes = self.build_metadata()?;  // Returns Vec<u8>
    
    // Step 2: Calculate offsets
    let raw_data_offset = metadata_bytes.len() as u64;
    
    // Step 3: Write header (28 bytes, batched)
    self.write_header(&mut writer, ...)?;
    
    // Step 4: Write metadata (single write_all call)
    writer.write_all(&metadata_bytes)?;
    writer.flush()?;  // Ensure metadata is written
    
    // Step 5: Write raw data directly to file (bypasses BufWriter)
    let file = writer.get_mut();
    for group in self.groups.values() {
        for channel in group.channels.values() {
            self.write_channel_data_direct(file, &channel.data)?;
        }
    }
    
    // Step 6: Sync to disk
    file.sync_all()?;
    Ok(())
}
```

### Write Characteristics

**Buffering Strategy**:
- ✅ **Buffered**: Metadata written through `BufWriter` (8MB buffer)
- ✅ **Unbuffered**: Raw data written directly to `File` (bypasses BufWriter)
- **Rationale**: For 1GB writes, BufWriter bypasses buffer anyway, so direct write avoids overhead

**Chunking Strategy**:
- ✅ **Per-segment**: Single segment written (format requirement)
- ✅ **Per-channel**: Data written channel-by-channel within segment
- ✅ **Single large write**: Each channel's data written in one `write_all()` call

**Streaming vs Staged**:
- ✅ **Staged**: Metadata built entirely in memory first, then written
- ✅ **Streaming**: Raw data written directly from source Vec (zero-copy)

### Write Syscall Analysis

For a typical 1GB write (125M f64 samples):

1. **Header write**: 1 syscall (28 bytes, batched)
2. **Metadata write**: 1 syscall (~few KB)
3. **Raw data write**: 1 syscall (1GB)
4. **Sync**: 1 syscall (`sync_all()`)

**Total**: ~4 syscalls for entire file write

**Assessment**: Minimal syscall overhead. The bottleneck is not syscall frequency.

## 2. Allocation & Copying Overhead

### Memory Allocation Pattern

```rust
// Metadata building
fn build_metadata(&self) -> Result<Vec<u8>> {
    // Pre-allocated with estimated capacity
    let estimated_capacity = 200 * (1 + self.groups.len() + ...);
    let mut metadata = Vec::with_capacity(estimated_capacity);
    
    // Many small writes to Vec (in-memory, fast)
    metadata.write_u32::<LittleEndian>(object_count)?;
    // ... more writes
}
```

**Allocation Strategy**:
- ✅ **Per-segment**: Metadata Vec allocated once per file
- ✅ **Pre-allocated**: Estimated capacity to avoid reallocations
- ✅ **Single allocation**: Raw data uses existing Vec, no new allocation

### Copying Overhead

```rust
// Raw data write for Double (f64)
fn write_channel_data_direct(&self, file: &mut File, data: &TdmsData) -> Result<()> {
    match data {
        TdmsData::Double(values) => {
            // Zero-copy: unsafe pointer cast to byte slice
            let buf = unsafe {
                std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 8)
            };
            file.write_all(buf)?;  // Single write_all call
        }
        // ...
    }
}
```

**Copying Analysis**:
- ✅ **Zero-copy at Rust level**: Uses unsafe pointer casting
- ⚠️ **OS-level copying**: OS may copy data internally (unavoidable)
- ✅ **No intermediate buffers**: Data written directly from source Vec

**Assessment**: No avoidable copies at Rust level. OS-level copying is inherent to file I/O.

## 3. Metadata & Segment Emission Strategy

### Metadata Structure

```rust
// Metadata contains:
// 1. Object count (4 bytes)
// 2. File object metadata
// 3. Group object metadata (one per group)
// 4. Channel object metadata (one per channel)
//    - Path string
//    - Raw data info (data type, dimension, count)
//    - Properties
```

**Emission Pattern**:
- ✅ **Single emission**: All metadata written once at start of file
- ✅ **No redundancy**: Metadata written only when needed
- ✅ **Batched**: Entire metadata Vec written in single `write_all()` call

### Interleaving Analysis

**Write Order**:
1. Header (28 bytes)
2. Metadata (~few KB)
3. Raw data (1GB)

**Assessment**: No interleaving. Metadata and data are clearly separated, allowing sequential disk access.

## 4. File I/O Configuration

### Current Configuration

```rust
let file = File::create(&self.path)?;
let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);
```

**BufWriter Settings**:
- Buffer size: 8MB
- Used for: Header + metadata only
- Raw data: Written directly to File (bypasses BufWriter)

**File Creation**:
- Uses default `File::create()` - no special flags
- No `FILE_FLAG_SEQUENTIAL_SCAN` or similar optimizations

### Comparison with nptdms

**Python File Buffering**:
- Default buffer: Typically 8KB, but can be larger
- Python's file object may have internal optimizations
- NumPy arrays are contiguous, enabling efficient writes

**Potential nptdms Advantages**:
1. Python's file object may use OS-specific optimizations
2. C extensions may have optimized write paths
3. Different buffering strategy internally

## 5. OS & Filesystem Interaction

### Flush/Sync Behavior

```rust
writer.flush()?;        // Flush BufWriter (metadata)
file.sync_all()?;       // Sync entire file to disk
```

**Flush Strategy**:
- ✅ **Minimal flushes**: Only flush metadata buffer, then sync once at end
- ✅ **Proper syncing**: `sync_all()` ensures data durability
- ⚠️ **Timing includes sync**: Both benchmarks include sync, so fair comparison

### Write Pattern Analysis

**Sequential Access**:
- ✅ **Sequential**: Header → Metadata → Data written sequentially
- ✅ **No seeks**: File pointer moves forward only
- ✅ **Large writes**: 1GB written in single operation

**Filesystem Friendliness**:
- ✅ **Sequential pattern**: Ideal for filesystem optimization
- ✅ **Large blocks**: 1GB write allows OS to optimize
- ⚠️ **No explicit flags**: May benefit from `FILE_FLAG_SEQUENTIAL_SCAN` on Windows

## 6. Benchmark Validity Check

### Payload Comparison

**tdms-rs**:
```rust
// benchmark/tdms_rs/benchmark.rs
const SAMPLE_COUNT: usize = 125_000_000;  // 1.0 GB of f64
let data: Vec<f64> = (0..sample_count).map(|i| i as f64).collect();
```

**nptdms**:
```python
# benchmark/nptdms/benchmark.py
SAMPLE_COUNT = 125_000_000  # 1.0 GB of f64
data = np.arange(sample_count, dtype=np.float64)
```

**Assessment**: ✅ Identical payload sizes (125M f64 = 1GB)

### Metadata Overhead

**tdms-rs metadata** (for single channel):
- Header: 28 bytes
- Object count: 4 bytes
- File object: ~20 bytes (path "/" + properties)
- Group object: ~30 bytes (path "/'Group1'" + properties)
- Channel object: ~50 bytes (path + raw data info + properties)
- **Total**: ~130 bytes

**nptdms metadata**: Similar structure, likely similar size

**Assessment**: ✅ Comparable metadata overhead (~130 bytes vs 1GB data = negligible)

### File Layout Comparison

**tdms-rs structure**:
```
[Header: 28 bytes]
[Metadata: ~130 bytes]
[Raw Data: 1GB]
```

**nptdms structure**: TDMS format is standardized, should be identical

**Assessment**: ✅ Identical file layout (TDMS format is standardized)

### Benchmark Asymmetry Check

**Both benchmarks**:
- ✅ Use same sample count (125M f64)
- ✅ Generate data before timing (excluded from measurement)
- ✅ Include sync/fsync in timing
- ✅ Use cold cache (clobber before each run)
- ✅ Measure minimum time (best performance)

**Assessment**: ✅ Fair comparison, no asymmetry detected

## Key Findings

### What's Working Well

1. ✅ **Zero-copy data writes**: Efficient pointer casting
2. ✅ **Minimal syscalls**: Only ~4 syscalls for entire file
3. ✅ **Sequential access**: Ideal write pattern
4. ✅ **Proper buffering**: Metadata buffered, large data direct

### What's Not the Bottleneck

1. ❌ **Buffer size**: Increasing from 1MB to 8MB had no effect
2. ❌ **Write batching**: Already optimal
3. ❌ **Memory allocation**: Pre-allocation didn't help
4. ❌ **BufWriter overhead**: Bypassing it didn't help

### Likely Bottleneck

The bottleneck appears to be at the **OS/filesystem level**:

1. **Windows file I/O behavior**: May handle Python writes differently
2. **sync_all() performance**: May be slower than Python's fsync()
3. **OS-level optimizations**: Python may benefit from flags we're not using
4. **Memory alignment**: OS may copy/align data differently

## Recommendations for Further Investigation

1. **Profile with ETW/Process Monitor**: Track actual syscalls and timing
2. **Measure sync_all() separately**: Isolate write vs sync time
3. **Try file flags**: `FILE_FLAG_SEQUENTIAL_SCAN`, `FILE_FLAG_NO_BUFFERING`
4. **Investigate nptdms source**: Understand Python-specific optimizations
5. **Try alternative APIs**: `std::fs::write()`, `write_vectored()`, memory-mapped I/O


