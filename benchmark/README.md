# TDMS Benchmarking Suite

This directory contains the unified benchmarking infrastructure for `tdms-rs` and `nptdms`.

## Structure

- `run.py`: The main orchestrator. Runs all benchmarks and generates reports.
- `config.yaml`: Configuration for file sizes, iteration counts, and paths.
- `disk/`: Raw disk benchmark logic (wraps `diskspd`).
- `nptdms/`: Python benchmark scripts using `nptdms` and `numpy`.
- `tdms_rs/`: Rust benchmark source code.
- `results/`: Generated reports (`summary.md`, `results.json`).
- `data/`: Temporary data files generated during benchmarking.

## Usage

Run the full benchmark suite from the repository root:

```bash
python benchmark/run.py
```

## Methodology

### 1. Cold-Cache via Memory Clobbering
To ensure a fair comparison and avoid OS page-cache effects, we standardized all benchmarks to use memory clobbering. Before every measured iteration:
- We allocate a buffer 4× the file size (~4 GB).
- We touch one byte per 4 KB page to ensure physical pages are mapped.
- We then drop the buffer. This pressure forces the OS to evict the TDMS file from the page cache.

### 2. Raw Disk Baseline
We measure raw sequential read and write performance of the disk.
- Tool: `diskspd` (Windows).
- Settings: OS caching enabled (but evicted via clobber), 1MB block size, sequential access.
- Units: All throughput is normalized to decimal GB/s ($10^9$ bytes/s).

### 3. File Size
Default: 1.0 GB ($1,000,000,000$ bytes).
This exceeds typical CPU L3 cache sizes to test main memory/disk throughput.

### 4. Measurements
- **Best-Time (Min):** We run multiple iterations (default 5 + 1 warmup) and take the *minimum* time (maximum speed).
- **Percentages:** Library performance is expressed as a percentage of raw disk bandwidth.
- **Storage:** Writes are explicitly synced to disk (`fsync` / `sync_all`) before timing ends.

## Requirements

- Python 3.x
- Rust (Cargo)
- `nptdms`, `numpy` (Python packages)
- `diskspd` (Windows only, expected in PATH)

## Interpreting Results

- **> 100%:** May indicate OS caching effects (if not strictly disabled) or compression (if enabled, currently disabled).
- **< 100%:** Indicates library overhead (serialization, buffer management, UTF-8 validation, etc.).
