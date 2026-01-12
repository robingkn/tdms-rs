# TDMS-RS Write Performance Analysis

## Benchmark Results (Reproduced)

```
Operation  | Disk (GB/s)  | nptdms          | tdms-rs        
------------------------------------------------------------
Write      | 0.72         | 0.59 (81.9%) | 0.47 (64.8%)
Read       | 1.83         | 1.32 (72.3%) | 2.08 (113.8%)
```

**Key Finding**: tdms-rs write throughput is **20% slower** than nptdms (0.47 vs 0.59 GB/s), despite outperforming on reads.

## Write Path Architecture Analysis

### Current tdms-rs Implementation

#### 1. Write Flow (`src/writer.rs::write()`)
```rust
1. Create File::create()
2. Wrap in BufWriter::with_capacity(1MB)
3. Build metadata into Vec<u8> (in-memory)
4. Write header (28 bytes) - multiple small writes
5. Write metadata Vec all at once
6. Write raw data channel-by-channel
7. flush() + sync_all()
```

#### 2. Metadata Building (`build_metadata()`)
- Builds entire metadata into `Vec<u8>` before writing
- Uses many small `write_u32`/`write_u64` calls to Vec (in-memory, fast)
- Metadata size: ~few KB for single channel
- **Assessment**: Efficient, not a bottleneck

#### 3. Raw Data Writing (`write_channel_data()`)
For `Double` (f64) data:
```rust
let buf = unsafe {
    std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 8)
};
writer.write_all(buf)?;
```
- Uses unsafe pointer casting (zero-copy)
- Single `write_all()` call per channel
- For 1GB data: single 1GB write to BufWriter
- **Assessment**: Should be efficient, but...

#### 4. Buffering Strategy
- `BufWriter` with 1MB buffer
- Sequential writes should benefit from buffering
- Large writes (>1MB) bypass buffer and go directly to OS

### Potential Bottlenecks Identified

#### Issue #1: BufWriter Buffer Size
**Problem**: 1MB buffer may be suboptimal for 1GB writes
- For writes >1MB, BufWriter bypasses buffer and calls OS directly
- Multiple large writes might benefit from larger buffer or different strategy
- **Impact**: Medium-High
- **Fix Complexity**: Low

#### Issue #2: Multiple Small Writes in Header
**Problem**: Header writes are small (4+4+8+8 bytes)
- Multiple `write_u32`/`write_u64` calls through BufWriter
- Could be batched into single write
- **Impact**: Low (header is tiny, but adds syscall overhead)
- **Fix Complexity**: Low

#### Issue #3: Metadata Write Pattern
**Problem**: Metadata written as single `write_all(&metadata_bytes)`
- This is actually good - single write
- But metadata Vec might be reallocated during building
- **Impact**: Low
- **Fix Complexity**: Low

#### Issue #4: No Pre-allocation for Metadata Vec
**Problem**: `Vec::new()` grows dynamically during metadata building
- Many small writes cause reallocations
- Could pre-allocate with estimated size
- **Impact**: Low-Medium
- **Fix Complexity**: Low

#### Issue #5: BufWriter Flush Behavior
**Problem**: `BufWriter` may flush on large writes
- For writes > buffer size, data goes directly to OS
- OS may not batch these efficiently
- **Impact**: Medium
- **Fix Complexity**: Medium

#### Issue #6: sync_all() Timing
**Problem**: `sync_all()` forces all data to disk
- This is correct for correctness, but timing includes it
- Both benchmarks include sync, so fair comparison
- **Impact**: None (both do it)
- **Fix Complexity**: N/A

## Comparison with nptdms

### nptdms Write Pattern (from benchmark.py)
```python
with TdmsWriter(filename) as tdms_writer:
    channel = ChannelObject("Group1", "Channel1", data)
    tdms_writer.write_segment([channel])
    tdms_writer._file.flush()
    os.fsync(tdms_writer._file.fileno())
```

**Key Differences**:
1. Python's file buffering (typically 8KB default, but can be larger)
2. NumPy arrays are contiguous in memory (like Rust Vec)
3. nptdms may use different buffering strategy internally
4. Python's C extensions may have optimized write paths

## Proposed Optimizations

### Optimization #1: Increase BufWriter Buffer Size
**Expected Impact**: +5-15% throughput
**Complexity**: Low
**Risk**: Low
**Implementation**: Change `BufWriter::with_capacity(1024 * 1024)` to larger size (e.g., 8MB or 16MB)

**Rationale**: Larger buffer allows OS to batch more writes, reducing syscall overhead.

### Optimization #2: Batch Header Writes
**Expected Impact**: +1-2% throughput
**Complexity**: Low
**Risk**: Low
**Implementation**: Build header into small buffer, write all at once

**Rationale**: Reduces number of small writes, though impact is minimal.

### Optimization #3: Pre-allocate Metadata Vec
**Expected Impact**: +1-3% throughput
**Complexity**: Low
**Risk**: Low
**Implementation**: Estimate metadata size, pre-allocate Vec with capacity

**Rationale**: Reduces reallocations during metadata building.

### Optimization #4: Use Direct File Writes for Large Data
**Expected Impact**: +10-20% throughput
**Complexity**: Medium
**Risk**: Medium
**Implementation**: For data > threshold (e.g., 8MB), write directly to File instead of through BufWriter

**Rationale**: BufWriter overhead for very large writes may not be worth it.

### Optimization #5: Write Raw Data Before Metadata (if format allows)
**Expected Impact**: Unknown
**Complexity**: High
**Risk**: High
**Implementation**: Would require format changes
**Status**: Not recommended - format constraints

## Testing Strategy

1. Implement optimizations incrementally
2. Run benchmark after each change
3. Measure both write and read performance (ensure no regressions)
4. Compare against baseline

## Optimization Results

### Implemented Optimizations

1. ✅ **Increased BufWriter buffer from 1MB to 8MB**
   - Result: No improvement (still 0.47 GB/s)
   - Analysis: For writes >8MB, BufWriter bypasses buffer anyway

2. ✅ **Batched header writes into single buffer**
   - Result: No measurable improvement
   - Analysis: Header is only 28 bytes, negligible impact

3. ✅ **Pre-allocated metadata Vec**
   - Result: No measurable improvement  
   - Analysis: Metadata is tiny (~few KB), not a bottleneck

4. ✅ **Direct file writes for raw data (bypassing BufWriter)**
   - Result: No improvement (still 0.47 GB/s)
   - Analysis: Direct writes don't help, suggesting bottleneck is elsewhere

### Key Finding

**None of the expected optimizations improved write performance**, indicating the bottleneck is not in:
- Buffer size
- Write batching
- Memory allocation
- BufWriter overhead

## Root Cause Hypothesis

The bottleneck is likely in one of these areas:

### Hypothesis #1: OS-Level Write Behavior
- Rust's `File::write_all()` for 1GB may trigger different OS behavior than Python
- Windows may handle Python's file writes differently (possibly due to FILE_FLAG_SEQUENTIAL_SCAN or other flags)
- **Investigation needed**: Compare syscall patterns using Process Monitor or similar

### Hypothesis #2: Memory Alignment/Copying
- The unsafe pointer cast may not be optimal for OS write operations
- OS may be copying data internally despite zero-copy at Rust level
- **Investigation needed**: Profile with perf/wt or similar tools

### Hypothesis #3: sync_all() Behavior
- Rust's `sync_all()` may be slower than Python's `fsync()`
- Both are included in timing, but implementation differences matter
- **Investigation needed**: Time sync_all separately

### Hypothesis #4: nptdms Internal Optimizations
- nptdms may use C extensions with optimized write paths
- Python's file object may have internal buffering we're not aware of
- **Investigation needed**: Examine nptdms source code

## Recommended Next Steps

1. **Profile with detailed instrumentation**
   - Use `perf` (Linux) or ETW (Windows) to measure syscall frequency
   - Measure time spent in `write_all()` vs `sync_all()`
   - Compare syscall patterns between Rust and Python

2. **Investigate nptdms implementation**
   - Check if nptdms uses special file flags or buffering
   - See if it writes data in a different pattern

3. **Try alternative write strategies**
   - Use `std::fs::write()` for entire raw data section
   - Try `write_vectored()` if metadata + data can be combined
   - Consider memory-mapped I/O for very large files

4. **Measure sync_all() separately**
   - Time write vs sync separately to isolate bottleneck
   - Compare Rust sync_all() vs Python fsync() performance

