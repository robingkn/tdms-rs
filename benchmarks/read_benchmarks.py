"""
TDMS Read Performance Benchmarks

Comprehensive benchmarks for TDMS file reading operations using nptdms.
These benchmarks measure the core read performance that will be compared
against the Rust implementation.

Rust Equivalent Operations:
- nptdms.TdmsFile.read() -> TdmsFile::load()
- file['group']['channel'] -> file.get_channel('group', 'channel')
- channel[:] -> channel.as_f64() / channel.as_i32() / etc.
- len(channel) -> channel.data_len()
"""

import numpy as np
from pathlib import Path
import time
import gc
from typing import List, Optional

try:
    import nptdms
except ImportError:
    print("Error: nptdms not installed. Run: pip install nptdms")
    exit(1)

from benchmark_utils import (
    benchmark_context, get_benchmark_results, clear_benchmark_results,
    write_csv_results, get_file_size_mb, ensure_test_files_exist
)


def benchmark_file_opening(test_files_dir: Path):
    """
    Benchmark TDMS file opening and metadata parsing.
    
    This measures the cost of:
    - File format validation
    - Metadata parsing
    - Object structure creation
    
    Rust equivalent: TdmsFile::load() - metadata parsing phase
    """
    print("Benchmarking file opening...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]  # Extract type from filename
        
        # Cold open (clear any OS caches as much as possible)
        gc.collect()
        
        with benchmark_context(
            "read_file_open",
            file_type,
            0,  # Unknown until opened
            0,  # Unknown until opened
            "metadata",
            "open_file",
            file_size_mb,
            "File opening and metadata parsing only"
        ):
            tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
            
            # Count channels for result recording
            total_channels = sum(len(group.channels()) for group in tdms_file_obj.groups())
            total_samples = 0
            
            # Update result with actual counts (hack: modify last result)
            if hasattr(benchmark_context, '_results') and benchmark_context._results:
                last_result = benchmark_context._results[-1]
                last_result.channels = total_channels
                
                # Estimate total samples (expensive to calculate exactly)
                for group in tdms_file_obj.groups():
                    for channel in group.channels():
                        if hasattr(channel, 'data') and channel.data is not None:
                            total_samples += len(channel.data)
                            break  # Just sample one channel per group for estimation
                
                last_result.samples = total_samples


def benchmark_single_channel_reads(test_files_dir: Path):
    """
    Benchmark single channel read operations.
    
    Tests different access patterns:
    - Full channel read
    - Slice reads (first N, middle, last N)
    - Random access
    
    Rust equivalent: channel.as_f64() and similar type-specific accessors
    """
    print("Benchmarking single channel reads...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Open file once for all channel tests
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find a suitable channel for testing
        test_channel = None
        test_group_name = None
        test_channel_name = None
        
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None and len(channel.data) > 100:
                    test_channel = channel
                    test_group_name = group.name
                    test_channel_name = channel.name
                    break
            if test_channel:
                break
        
        if not test_channel:
            continue  # Skip files with no suitable channels
        
        channel_len = len(test_channel.data)
        data_type = type(test_channel.data[0]).__name__ if channel_len > 0 else "unknown"
        
        # Full channel read
        with benchmark_context(
            "read_single_channel",
            file_type,
            1,
            channel_len,
            data_type,
            "full_read",
            file_size_mb,
            f"Full read of channel {test_group_name}/{test_channel_name}"
        ):
            full_data = test_channel[:]
            # Force evaluation
            _ = len(full_data)
        
        # Slice reads (if channel is large enough)
        if channel_len > 1000:
            # First 100 samples
            with benchmark_context(
                "read_single_channel",
                file_type,
                1,
                100,
                data_type,
                "slice_first",
                file_size_mb * (100 / channel_len),
                "Read first 100 samples"
            ):
                first_data = test_channel[:100]
                _ = len(first_data)
            
            # Middle slice
            start = channel_len // 2 - 50
            end = channel_len // 2 + 50
            with benchmark_context(
                "read_single_channel",
                file_type,
                1,
                100,
                data_type,
                "slice_middle",
                file_size_mb * (100 / channel_len),
                f"Read middle slice [{start}:{end}]"
            ):
                middle_data = test_channel[start:end]
                _ = len(middle_data)
            
            # Last 100 samples
            with benchmark_context(
                "read_single_channel",
                file_type,
                1,
                100,
                data_type,
                "slice_last",
                file_size_mb * (100 / channel_len),
                "Read last 100 samples"
            ):
                last_data = test_channel[-100:]
                _ = len(last_data)
        
        # Random access (if channel is large enough)
        if channel_len > 10000:
            indices = np.random.choice(channel_len, size=min(1000, channel_len // 10), replace=False)
            with benchmark_context(
                "read_single_channel",
                file_type,
                1,
                len(indices),
                data_type,
                "random_access",
                file_size_mb * (len(indices) / channel_len),
                f"Random access to {len(indices)} samples"
            ):
                for idx in indices:
                    _ = test_channel[idx]


def benchmark_multi_channel_reads(test_files_dir: Path):
    """
    Benchmark multi-channel read operations.
    
    Tests:
    - Reading all channels in a group
    - Reading all channels in file
    - Selective channel reading
    
    Rust equivalent: Multiple calls to channel.as_f64() etc.
    """
    print("Benchmarking multi-channel reads...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Read all channels in file
        total_channels = 0
        total_samples = 0
        
        with benchmark_context(
            "read_multi_channel",
            file_type,
            0,  # Will be updated
            0,  # Will be updated
            "mixed",
            "all_channels",
            file_size_mb,
            "Read all channels in file"
        ):
            all_data = {}
            for group in tdms_file_obj.groups():
                for channel in group.channels():
                    if hasattr(channel, 'data') and channel.data is not None:
                        all_data[f"{group.name}/{channel.name}"] = channel[:]
                        total_channels += 1
                        total_samples += len(channel.data)
        
        # Update result with actual counts
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.channels = total_channels
            last_result.samples = total_samples
        
        # Read channels by group (if multiple groups exist)
        groups = list(tdms_file_obj.groups())
        if len(groups) > 1:
            for group in groups[:3]:  # Test first 3 groups max
                group_channels = 0
                group_samples = 0
                
                with benchmark_context(
                    "read_multi_channel",
                    file_type,
                    0,  # Will be updated
                    0,  # Will be updated
                    "mixed",
                    "group_channels",
                    file_size_mb / len(groups),  # Approximate
                    f"Read all channels in group {group.name}"
                ):
                    group_data = {}
                    for channel in group.channels():
                        if hasattr(channel, 'data') and channel.data is not None:
                            group_data[channel.name] = channel[:]
                            group_channels += 1
                            group_samples += len(channel.data)
                
                # Update result
                if hasattr(benchmark_context, '_results') and benchmark_context._results:
                    last_result = benchmark_context._results[-1]
                    last_result.channels = group_channels
                    last_result.samples = group_samples


def benchmark_repeated_reads(test_files_dir: Path):
    """
    Benchmark repeated access to the same data (warm cache).
    
    This tests:
    - Python object caching behavior
    - Memory vs computation tradeoffs
    - Repeated access patterns
    
    Rust equivalent: Multiple calls to the same channel accessor
    """
    print("Benchmarking repeated reads...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Skip very large files for repeated access tests
        if file_size_mb > 100:
            continue
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find a suitable channel
        test_channel = None
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None and len(channel.data) > 0:
                    test_channel = channel
                    break
            if test_channel:
                break
        
        if not test_channel:
            continue
        
        channel_len = len(test_channel.data)
        data_type = type(test_channel.data[0]).__name__ if channel_len > 0 else "unknown"
        
        # First read (cold)
        with benchmark_context(
            "read_repeated",
            file_type,
            1,
            channel_len,
            data_type,
            "first_read",
            file_size_mb,
            "First read (cold cache)"
        ):
            first_data = test_channel[:]
            _ = len(first_data)
        
        # Repeated reads (warm cache)
        for i in range(3):
            with benchmark_context(
                "read_repeated",
                file_type,
                1,
                channel_len,
                data_type,
                f"repeat_{i+1}",
                file_size_mb,
                f"Repeated read #{i+1} (warm cache)"
            ):
                repeat_data = test_channel[:]
                _ = len(repeat_data)


def benchmark_numpy_conversion(test_files_dir: Path):
    """
    Benchmark NumPy array conversion costs.
    
    This measures the overhead of converting TDMS data to NumPy arrays,
    which is a common operation in Python but not directly applicable to Rust.
    
    Note: This benchmark is Python-specific and shows the conversion overhead
    that Rust implementations don't have.
    """
    print("Benchmarking NumPy conversion...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Skip very large files
        if file_size_mb > 50:
            continue
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None and len(channel.data) > 0:
                    channel_len = len(channel.data)
                    data_type = type(channel.data[0]).__name__
                    
                    # Get raw data first
                    raw_data = channel[:]
                    
                    # Convert to NumPy array
                    with benchmark_context(
                        "read_numpy_conversion",
                        file_type,
                        1,
                        channel_len,
                        data_type,
                        "to_numpy",
                        file_size_mb,
                        "Convert channel data to NumPy array"
                    ):
                        numpy_array = np.array(raw_data)
                        _ = numpy_array.shape  # Force evaluation
                    
                    # Only test first channel per group to avoid too many results
                    break


def main():
    """Run all read benchmarks."""
    print("Starting TDMS Read Benchmarks")
    print("=" * 50)
    
    # Ensure test files exist
    ensure_test_files_exist()
    
    test_files_dir = Path(__file__).parent / "test_files"
    clear_benchmark_results()
    
    # Run all benchmark categories
    benchmark_file_opening(test_files_dir)
    benchmark_single_channel_reads(test_files_dir)
    benchmark_multi_channel_reads(test_files_dir)
    benchmark_repeated_reads(test_files_dir)
    benchmark_numpy_conversion(test_files_dir)
    
    # Save results
    results = get_benchmark_results()
    output_dir = Path(__file__).parent / "results"
    write_csv_results(results, output_dir / "read_benchmarks.csv")
    
    print(f"\nCompleted {len(results)} read benchmarks")
    print(f"Results saved to: {output_dir / 'read_benchmarks.csv'}")


if __name__ == "__main__":
    main()