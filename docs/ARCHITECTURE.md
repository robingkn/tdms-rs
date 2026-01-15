# tdms-rs Architecture

## Overview
`tdms-rs` is designed as a high-performance, type-safe Rust library for the National Instruments TDMS format. It prioritizes memory efficiency (especially for large datasets) and ergonomic API usage.

## Reader Model
The reader uses an indexed approach. When a file is opened:
1.  The library performs a linear scan of all segments.
2.  Metadata is parsed into an in-memory index (`IndexMap`).
3.  Raw data locations (offsets and lengths) are stored without loading the actual data.

### Lazy Loading & Zero-Copy
- **Metadata**: Loaded eagerly to provide fast navigation.
- **Raw Data**: Loaded lazily when `read()` or `read_into()` is called.
- **Zero-Copy**: The structure supports memory-mapping (`mmap`) for high-throughput reads, though owned buffers are used as a fallback.

## Writer Model
The writer uses a staged approach to optimize for disk I/O throughput:
1.  **Metadata Staging**: All metadata (groups, channels, properties) is built in memory first.
2.  **Single Segment Emission**: `tdms-rs` currently writes data in a single large segment to maximize sequential write performance.
3.  **Buffering**: 
    - Metadata is written through an 8MB `BufWriter` to batch small syscalls.
    - Raw data is written directly to the file handle, bypassing the user-space buffer for large transfers.

### Write Performance Insights
From performance audits, the primary bottleneck for TDMS writes is often the OS-level sync or filesystem overhead rather than the library's serialization logic. `tdms-rs` achieves near-disk bandwidth by:
- Using `unsafe` pointer casting for zero-copy serialization of numeric slices.
- Minimizing syscalls (typically ~4 syscalls for a 1GB file write).

## Ownership & Buffer Responsibility
- `TdmsFile` owns the file handle and the index.
- `TdmsChannel` and `TdmsGroup` are lightweight views into the index.
- `TdmsSlice` manages the lifecycle of returned data (whether owned or mapped).

## Performance Tradeoffs
- **In-memory Index**: For files with millions of objects, memory usage for the index may be significant.
- **Single Segment Writing**: While fast, it requires knowing the data schema/size beforehand or buffering data in memory.
