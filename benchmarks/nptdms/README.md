# Raw Disk vs nptdms Benchmark

This benchmark measures the sequential read performance of a raw disk (via `diskspd`) and compares it to the full-channel read performance of `nptdms` in Python.

## Prerequisites

### Python Dependencies
```bash
pip install numpy nptdms
```

### System Tools
- **Windows**: [diskspd](https://github.com/microsoft/diskspd) is required.
    1. Download `DiskSpd.zip` from [GitHub Releases](https://github.com/microsoft/diskspd/releases).
    2. Extract it.
    3. Copy `diskspd.exe` (use the version matching your architecture, e.g., `amd64fre/diskspd.exe` for 64-bit) into this directory OR ensure it is in your system PATH.

## Usage

1. **Generate the Test File**
   Create a ~512MB TDMS file composed of float64 data.
   ```bash
   python generate_file.py
   ```

2. **Run the Benchmark**
   Execute the benchmark script.
   ```bash
   python benchmark.py
   ```

## Methodology

### Part 1: Raw Disk Baseline
Uses `diskspd` with unbuffered I/O (flag `-Sh`) to bypass OS file cache and measure the drive's true physical sequential read speed.
- Block Size: 1MB
- Duration: 10s
- Thread: 1
- Queue Depth: 1

### Part 2: nptdms Performance
1. Opens the generated TDMS file.
2. Performs 5 full-channel reads (`channel[:]`).
3. Discards the first run (warm-up).
4. Records the **minimum time** of the remaining runs to determine peak throughput.
5. Touches data (sum) to ensure evaluation.

### Interpretation
- If **nptdms > Raw Disk**: The file was likely cached in RAM by the OS. The result represents the maximum CPU-bound parsing speed of `nptdms`.
- If **nptdms < Raw Disk**: The bottleneck is likely `nptdms` parsing overhead or Python execution speed (or the file was not cached and disk I/O was the limit, effectively utilizing < 100% of bandwidth).
