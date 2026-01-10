"""
TDMS Write Performance Benchmarks

Comprehensive benchmarks for TDMS file writing operations using nptdms.
These benchmarks measure write performance patterns that will be compared
against the Rust implementation.

Rust Equivalent Operations:
- TdmsWriter() -> TdmsFileWriter::new()
- ChannelObject() -> group.add_channel()
- write_data() -> writer.write()
- Properties -> add_property() calls
"""

import numpy as np
from pathlib import Path
import tempfile
import os
from typing import List, Dict, Any

try:
    import nptdms
    from nptdms import TdmsWriter, ChannelObject
except ImportError:
    print("Error: nptdms not installed. Run: pip install nptdms")
    exit(1)

from benchmark_utils import (
    benchmark_context, get_benchmark_results, clear_benchmark_results,
    write_csv_results, get_file_size_mb
)


def benchmark_single_channel_writes():
    """
    Benchmark single channel write operations.
    
    Tests different data sizes and types for single channel writes.
    
    Rust equivalent: 
    - TdmsFileWriter::new() + group.add_channel() + writer.write()
    """
    print("Benchmarking single channel writes...")
    
    # Test different data sizes
    test_sizes = [
        (1000, "small"),
        (100000, "medium"), 
        (1000000, "large")
    ]
    
    # Test different data types
    data_types = [
        ("float64", np.float64, lambda n: np.random.random(n)),
        ("float32", np.float32, lambda n: np.random.random(n).astype(np.float32)),
        ("int32", np.int32, lambda n: np.random.randint(-1000, 1000, n).astype(np.int32)),
        ("int16", np.int16, lambda n: np.random.randint(-100, 100, n).astype(np.int16)),
        ("uint8", np.uint8, lambda n: np.random.randint(0, 255, n).astype(np.uint8)),
    ]
    
    for samples, size_category in test_sizes:
        for type_name, dtype, generator in data_types:
            with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
                tmp_path = tmp_file.name
            
            try:
                # Generate test data
                data = generator(samples)
                
                with benchmark_context(
                    "write_single_channel",
                    size_category,
                    1,
                    samples,
                    type_name,
                    "write_complete",
                    0,  # Will be calculated after write
                    f"Single channel write: {samples} {type_name} samples"
                ):
                    with TdmsWriter(tmp_path) as tdms_writer:
                        channel = ChannelObject('Data', 'TestChannel', data, properties={
                            'DataType': type_name,
                            'SampleCount': samples
                        })
                        tdms_writer.write_segment([channel])
                
                # Update file size in result
                file_size_mb = get_file_size_mb(Path(tmp_path))
                if hasattr(benchmark_context, '_results') and benchmark_context._results:
                    last_result = benchmark_context._results[-1]
                    last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
                
            finally:
                if os.path.exists(tmp_path):
                    os.unlink(tmp_path)


def benchmark_multi_channel_writes():
    """
    Benchmark multi-channel write operations.
    
    Tests writing multiple channels simultaneously vs incrementally.
    
    Rust equivalent: Multiple group.add_channel() calls + single writer.write()
    """
    print("Benchmarking multi-channel writes...")
    
    # Test different channel counts
    channel_configs = [
        (5, 10000, "few_channels"),
        (20, 5000, "many_channels"),
        (100, 1000, "very_many_channels")
    ]
    
    for num_channels, samples_per_channel, config_name in channel_configs:
        with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
            tmp_path = tmp_file.name
        
        try:
            with benchmark_context(
                "write_multi_channel",
                config_name,
                num_channels,
                samples_per_channel * num_channels,
                "float64",
                "bulk_write",
                0,  # Will be calculated after write
                f"Bulk write: {num_channels} channels, {samples_per_channel} samples each"
            ):
                with TdmsWriter(tmp_path) as tdms_writer:
                    channels = []
                    for i in range(num_channels):
                        data = np.random.random(samples_per_channel)
                        channel = ChannelObject('Data', f'Channel_{i:03d}', data, properties={
                            'ChannelIndex': i
                        })
                        channels.append(channel)
                    
                    tdms_writer.write_segment(channels)
            
            # Update file size in result
            file_size_mb = get_file_size_mb(Path(tmp_path))
            if hasattr(benchmark_context, '_results') and benchmark_context._results:
                last_result = benchmark_context._results[-1]
                last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
        
        finally:
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)


def benchmark_incremental_writes():
    """
    Benchmark incremental write operations.
    
    Tests writing channels one at a time vs all at once.
    Note: nptdms may not support true incremental writes,
    so this tests the overhead of multiple write operations.
    
    Rust equivalent: Multiple writer.write() calls vs single bulk write
    """
    print("Benchmarking incremental writes...")
    
    num_channels = 10
    samples_per_channel = 5000
    
    # Incremental approach (if supported by nptdms)
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        with benchmark_context(
            "write_incremental",
            "medium",
            num_channels,
            samples_per_channel * num_channels,
            "float64",
            "incremental_write",
            0,  # Will be calculated after write
            f"Incremental write: {num_channels} separate write operations"
        ):
            with TdmsWriter(tmp_path) as tdms_writer:
                for i in range(num_channels):
                    data = np.random.random(samples_per_channel)
                    channel = ChannelObject('Data', f'Channel_{i:03d}', data)
                    channel.properties['ChannelIndex'] = i
                    tdms_writer.write_data([channel])
        
        # Update file size in result
        file_size_mb = get_file_size_mb(Path(tmp_path))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def benchmark_property_overhead():
    """
    Benchmark the overhead of writing properties.
    
    Tests files with many properties vs files without properties.
    
    Rust equivalent: Multiple add_property() calls
    """
    print("Benchmarking property overhead...")
    
    samples = 10000
    
    # Write without properties
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path_no_props = tmp_file.name
    
    try:
        with benchmark_context(
            "write_properties",
            "medium",
            1,
            samples,
            "float64",
            "no_properties",
            0,
            "Write channel without properties"
        ):
            with TdmsWriter(tmp_path_no_props) as tdms_writer:
                data = np.random.random(samples)
                channel = ChannelObject('Data', 'TestChannel', data)
                tdms_writer.write_data([channel])
        
        file_size_mb = get_file_size_mb(Path(tmp_path_no_props))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
    
    finally:
        if os.path.exists(tmp_path_no_props):
            os.unlink(tmp_path_no_props)
    
    # Write with many properties
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path_with_props = tmp_file.name
    
    try:
        with benchmark_context(
            "write_properties",
            "medium",
            1,
            samples,
            "float64",
            "many_properties",
            0,
            "Write channel with 50 properties"
        ):
            with TdmsWriter(tmp_path_with_props) as tdms_writer:
                # Add file-level properties
                tdms_writer.file_properties = {
                    'Title': 'Property Overhead Test',
                    'Author': 'Benchmark Suite',
                    'Version': '1.0',
                    'Created': '2024-01-01T00:00:00Z',
                    'Description': 'Testing property write overhead'
                }
                
                data = np.random.random(samples)
                channel = ChannelObject('Data', 'TestChannel', data)
                
                # Add many channel properties
                for i in range(50):
                    channel.properties[f'Property_{i:02d}'] = f'Value_{i}'
                    channel.properties[f'Numeric_{i:02d}'] = i * 1.5
                    channel.properties[f'Boolean_{i:02d}'] = i % 2 == 0
                
                # Add standard TDMS properties
                channel.properties['wf_unit_string'] = 'V'
                channel.properties['wf_increment'] = 0.001
                channel.properties['Description'] = 'Test channel with many properties'
                
                tdms_writer.write_data([channel])
        
        file_size_mb = get_file_size_mb(Path(tmp_path_with_props))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
    
    finally:
        if os.path.exists(tmp_path_with_props):
            os.unlink(tmp_path_with_props)


def benchmark_mixed_data_types():
    """
    Benchmark writing files with mixed data types.
    
    Tests the overhead of handling multiple data types in one file.
    
    Rust equivalent: Multiple add_channel() calls with different TdmsData variants
    """
    print("Benchmarking mixed data type writes...")
    
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        samples_per_type = 5000
        
        with benchmark_context(
            "write_mixed_types",
            "medium",
            5,  # 5 different data types
            samples_per_type * 5,
            "mixed",
            "mixed_types",
            0,
            "Write channels with 5 different data types"
        ):
            with TdmsWriter(tmp_path) as tdms_writer:
                channels = []
                
                # Float64 channel
                data_f64 = np.random.random(samples_per_type)
                channel_f64 = ChannelObject('Mixed', 'Float64', data_f64)
                channel_f64.properties['DataType'] = 'float64'
                channels.append(channel_f64)
                
                # Float32 channel
                data_f32 = np.random.random(samples_per_type).astype(np.float32)
                channel_f32 = ChannelObject('Mixed', 'Float32', data_f32)
                channel_f32.properties['DataType'] = 'float32'
                channels.append(channel_f32)
                
                # Int32 channel
                data_i32 = np.random.randint(-1000, 1000, samples_per_type).astype(np.int32)
                channel_i32 = ChannelObject('Mixed', 'Int32', data_i32)
                channel_i32.properties['DataType'] = 'int32'
                channels.append(channel_i32)
                
                # Int16 channel
                data_i16 = np.random.randint(-100, 100, samples_per_type).astype(np.int16)
                channel_i16 = ChannelObject('Mixed', 'Int16', data_i16)
                channel_i16.properties['DataType'] = 'int16'
                channels.append(channel_i16)
                
                # UInt8 channel
                data_u8 = np.random.randint(0, 255, samples_per_type).astype(np.uint8)
                channel_u8 = ChannelObject('Mixed', 'UInt8', data_u8)
                channel_u8.properties['DataType'] = 'uint8'
                channels.append(channel_u8)
                
                tdms_writer.write_data(channels)
        
        file_size_mb = get_file_size_mb(Path(tmp_path))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def benchmark_large_writes():
    """
    Benchmark large file write operations.
    
    Tests writing very large amounts of data to measure sustained throughput.
    
    Rust equivalent: Large TdmsData variants in add_channel()
    """
    print("Benchmarking large writes...")
    
    # Large single channel
    large_samples = 1000000  # 1M samples
    
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        with benchmark_context(
            "write_large",
            "large",
            1,
            large_samples,
            "float64",
            "large_single",
            0,
            f"Large single channel: {large_samples} float64 samples"
        ):
            with TdmsWriter(tmp_path) as tdms_writer:
                data = np.random.random(large_samples)
                channel = ChannelObject('Data', 'LargeChannel', data)
                channel.properties['Description'] = 'Large channel for throughput testing'
                tdms_writer.write_data([channel])
        
        file_size_mb = get_file_size_mb(Path(tmp_path))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def main():
    """Run all write benchmarks."""
    print("Starting TDMS Write Benchmarks")
    print("=" * 50)
    
    clear_benchmark_results()
    
    # Run all benchmark categories
    benchmark_single_channel_writes()
    benchmark_multi_channel_writes()
    benchmark_incremental_writes()
    benchmark_property_overhead()
    benchmark_mixed_data_types()
    benchmark_large_writes()
    
    # Save results
    results = get_benchmark_results()
    output_dir = Path(__file__).parent / "results"
    write_csv_results(results, output_dir / "write_benchmarks.csv")
    
    print(f"\nCompleted {len(results)} write benchmarks")
    print(f"Results saved to: {output_dir / 'write_benchmarks.csv'}")


if __name__ == "__main__":
    main()