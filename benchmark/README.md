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

### 1. Raw Disk Baseline
We measure raw sequential read and write performance of the disk to establish a 100% baseline.
- Tool: `diskspd` (Windows).
- Settings: Unbuffered I/O, 1MB block size, sequential access.

### 2. File Size
Default: 1.0 GB (125,000,000 float64 samples).
This size is chosen to be large enough to minimize startup overhead but small enough to run quickly. It also exceeds typical CPU L3 cache sizes to test main memory/disk throughput.

### 3. Measurements
- **Best-Time (Min):** We run multiple iterations (default 5 + 1 warmup) and take the *minimum* time (maximum speed). This reduces noise from background system processes.
- **Percentages:** Library performance is expressed as a percentage of raw disk bandwidth.

## Requirements

- Python 3.x
- Rust (Cargo)
- `nptdms`, `numpy` (Python packages)
- `diskspd` (Windows only, expected in PATH)

## Interpreting Results

- **> 100%:** May indicate OS caching effects (if not strictly disabled) or compression (if enabled, currently disabled).
- **< 100%:** Indicates library overhead (serialization, buffer management, UTF-8 validation, etc.).
