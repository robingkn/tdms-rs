# TDMS-RS Write Performance Analysis Summary

## Executive Summary

**Problem**: tdms-rs write throughput (0.47 GB/s) is **22% slower** than nptdms (0.61 GB/s), despite outperforming on reads (2.17 vs 1.24 GB/s).

**Investigation**: Comprehensive analysis of write path architecture, buffering strategies, and I/O operations.

**Result**: Standard optimizations (buffer size, batching, direct writes) showed **no improvement**, indicating bottleneck is at OS/filesystem level or in nptdms-specific optimizations.

## Benchmark Results

```
Operation  | Disk (GB/s)  | nptdms          | tdms-rs        
------------------------------------------------------------
Write      | 0.72         | 0.61 (84.4%) | 0.47 (66.1%)
Read       | 1.64         | 1.24 (75.7%) | 2.17 (132.2%)
```

**Key Metrics**:
- Disk baseline: 0.72 GB/s write, 1.64 GB/s read
- tdms-rs achieves 66% of disk write speed vs nptdms's 84%
- Read performance: tdms-rs is 75% faster than nptdms

## Write Path Architecture

### tdms-rs Current Implementation

1. **File Creation**: `File::create()` → `BufWriter` (8MB buffer)
2. **Metadata Building**: Builds entire metadata into `Vec<u8>` (~few KB)
3. **Writing**:
   - Header (28 bytes) - batched into single write
   - Metadata - single `write_all()` call
   - Raw data - direct `File::write_all()` for 1GB chunk
4. **Sync**: `sync_all()` to ensure data on disk

### Key Characteristics

- **Zero-copy data writes**: Uses unsafe pointer casting for f64 arrays
- **Single large write**: 1GB written in one `write_all()` call
- **Minimal metadata**: ~few KB, written efficiently
- **Proper syncing**: Includes `sync_all()` for correctness

## Optimizations Attempted

### ✅ Optimization #1: Increased Buffer Size
- **Change**: 1MB → 8MB BufWriter buffer
- **Result**: No improvement
- **Reason**: BufWriter bypasses buffer for writes > buffer size

### ✅ Optimization #2: Batched Header Writes  
- **Change**: Header written as single 28-byte buffer
- **Result**: No measurable improvement
- **Reason**: Header is tiny, negligible impact

### ✅ Optimization #3: Pre-allocated Metadata Vec
- **Change**: Estimated capacity, pre-allocated Vec
- **Result**: No measurable improvement
- **Reason**: Metadata is small, not a bottleneck

### ✅ Optimization #4: Direct File Writes
- **Change**: Bypassed BufWriter for raw data writes
- **Result**: No improvement
- **Reason**: Direct writes don't help, bottleneck is elsewhere

## Root Cause Analysis

### Eliminated Causes

❌ **Buffer size** - Increased buffer had no effect  
❌ **Write batching** - Already optimal  
❌ **Memory allocation** - Pre-allocation didn't help  
❌ **BufWriter overhead** - Bypassing it didn't help  

### Likely Causes

1. **OS-Level Write Behavior** (Most Likely)
   - Windows may handle Python file writes differently
   - Possible FILE_FLAG_SEQUENTIAL_SCAN or other optimizations in Python
   - Rust's `File::write_all()` may trigger different code paths

2. **sync_all() Performance**
   - Rust's `sync_all()` may be slower than Python's `fsync()`
   - Both included in timing, but implementation differs
   - Need to measure sync time separately

3. **nptdms Internal Optimizations**
   - Python C extensions may use optimized write paths
   - Python file object may have internal buffering
   - NumPy arrays may have memory layout advantages

4. **Memory Alignment/Copying**
   - OS may copy data internally despite zero-copy at Rust level
   - Memory alignment may affect OS write performance

## Recommendations

### Immediate Actions

1. **Profile with instrumentation**
   - Use Windows ETW or Process Monitor to track syscalls
   - Measure time in `write_all()` vs `sync_all()`
   - Compare syscall patterns between Rust and Python

2. **Measure sync_all() separately**
   - Time write vs sync to isolate bottleneck
   - Compare Rust `sync_all()` vs Python `fsync()` performance

3. **Investigate nptdms source**
   - Check for special file flags or buffering strategies
   - Understand how Python file writes are optimized

### Potential Solutions

1. **Try alternative write APIs**
   - `std::fs::write()` for entire raw data section
   - `write_vectored()` if metadata + data can be combined
   - Memory-mapped I/O for very large files

2. **File flags optimization**
   - Use `OpenOptions` with `FILE_FLAG_SEQUENTIAL_SCAN` (Windows)
   - Set appropriate file flags for sequential writes

3. **Chunked writes** (if OS benefits)
   - Write in 64MB-128MB chunks instead of single 1GB write
   - May allow OS to optimize better

4. **Async I/O** (if applicable)
   - Use `tokio::fs` or similar for potentially better batching
   - May not help for single large sequential write

## Conclusion

The write performance gap is **not** due to:
- Buffer size
- Write batching  
- Memory allocation
- BufWriter overhead

The bottleneck appears to be at the **OS/filesystem level** or in **nptdms-specific optimizations**. Further investigation requires:
1. Detailed profiling with OS-level tools
2. Comparison of syscall patterns
3. Understanding nptdms's write implementation

**Read performance advantage** (75% faster) suggests tdms-rs's architecture is sound; the write gap is likely due to OS-level or Python-specific optimizations that need deeper investigation.


