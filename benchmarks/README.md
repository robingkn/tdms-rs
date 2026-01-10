# nptdms Benchmark Suite

This benchmark suite provides comprehensive, fair, and reproducible performance measurements for the Python `nptdms` library. These benchmarks serve as the baseline for comparing against the Rust implementation `tdms-rs`.

## Design Principles

- **Honest & Fair**: No artificial optimizations or misleading measurements
- **Reproducible**: Deterministic inputs and consistent methodology
- **Comprehensive**: Covers real-world usage patterns and edge cases
- **CI-Friendly**: Smoke tests run quickly, full suite available for deep analysis
- **Cross-Language Ready**: Results can be directly compared with Rust implementation

## Quick Start

### Prerequisites

```bash
pip install nptdms numpy psutil
```

### Run Smoke Tests (CI-Safe, ~5 minutes)

```bash
python run_benchmarks.py --mode smoke
```

### Run Full Benchmark Suite (~30-60 minutes)

```bash
python run_benchmarks.py --mode full
```

### Run Individual Benchmark Categories

```bash
python benchmarks/read_benchmarks.py
python benchmarks/write_benchmarks.py
python benchmarks/channel_access_benchmarks.py
python benchmarks/stress_benchmarks.py
```

## Benchmark Categories

### 1. Read Benchmarks (`read_benchmarks.py`)
- File opening and metadata parsing
- Single channel reads (full, sliced, random access)
- Multi-channel reads
- Cold vs warm cache performance
- Different file structures and sizes

### 2. Write Benchmarks (`write_benchmarks.py`)
- Single vs multi-channel writes
- Incremental vs bulk writes
- Different data types and sizes
- Property handling overhead

### 3. Channel Access Patterns (`channel_access_benchmarks.py`)
- Python abstraction overhead
- NumPy conversion costs
- Repeated access patterns
- Metadata-only operations

### 4. Stress Tests (`stress_benchmarks.py`)
- Thousands of channels
- Very large/small channels
- Mixed data types
- Large property dictionaries

## Output Format

All benchmarks emit CSV data with consistent schema:

```csv
benchmark_name,file_type,channels,samples,data_type,operation,time_sec,mb_per_sec,peak_memory_mb,notes
```

Optional JSON output for CI artifact storage.

## File Structure

```
benchmarks/
├── README.md                    # This file
├── run_benchmarks.py           # Main benchmark runner
├── benchmark_utils.py          # Shared utilities and helpers
├── generate_test_files.py      # Create benchmark TDMS files
├── read_benchmarks.py          # Read performance tests
├── write_benchmarks.py         # Write performance tests
├── channel_access_benchmarks.py # Channel access patterns
├── stress_benchmarks.py        # Pathological cases
├── test_files/                 # Generated benchmark files
└── results/                    # Benchmark output
```

## Test Files

The benchmark suite generates its own TDMS files to ensure:
- Deterministic inputs
- Known data patterns
- Scalable file sizes
- Reusable by Rust implementation

File categories:
- **Small**: 1-10 MB, few channels
- **Medium**: 100-500 MB, moderate complexity
- **Large**: 1-5 GB, many channels or large data
- **Structural variants**: Different group/channel layouts
- **Data type variants**: All TDMS data types

## Rust Comparison Notes

When porting these benchmarks to Rust (`tdms-rs`):

1. **File Operations**: `TdmsFile::load()` maps to `nptdms.TdmsFile.read()`
2. **Channel Access**: `file.get_channel()` maps to `file['group']['channel']`
3. **Data Access**: `channel.as_f64()` maps to `channel[:]` with type checking
4. **Property Access**: Direct property access maps to `channel.properties`

## Known Limitations

- **Python Overhead**: These benchmarks include Python interpreter overhead
- **Memory Model**: Python's garbage collection affects memory measurements
- **I/O Buffering**: OS-level caching may affect cold read measurements
- **nptdms Specifics**: Some measurements are specific to nptdms implementation

## Extending the Suite

To add new benchmarks:

1. Follow the naming convention: `{category}_benchmarks.py`
2. Use `benchmark_utils.py` for common functionality
3. Emit CSV output with the standard schema
4. Add documentation for Rust equivalents
5. Update `run_benchmarks.py` to include new benchmarks

## CI Integration

The benchmark suite supports two modes:

- **Smoke Mode**: Quick validation (~5 minutes)
  - Small files only
  - Basic operations
  - Regression detection
  
- **Full Mode**: Comprehensive analysis (~30-60 minutes)
  - All file sizes
  - All operations
  - Detailed profiling

## Quality Assurance

These benchmarks are designed to be:
- **Credible**: Acceptable to both Python and Rust developers
- **Unbiased**: No artificial advantages for either implementation
- **Realistic**: Based on actual TDMS usage patterns
- **Documented**: Clear about what is and isn't measured

## License

Same as parent project: MIT OR Apache-2.0