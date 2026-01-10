"""
TDMS Stress Test Benchmarks

Pathological and edge case benchmarks designed to test nptdms under
extreme conditions. These tests reveal performance characteristics
under stress and help identify bottlenecks.

Rust Equivalent Operations:
- Thousands of channels -> Many add_channel() calls
- Very large channels -> Large TdmsData variants
- Mixed data types -> Different TdmsData enum variants
- Many properties -> Many add_property() calls
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
    write_csv_results, get_file_size_mb, ensure_test_files_exist
)


def benchmark_many_channels_read():
    """
    Benchmark reading files with thousands of channels.
    
    Tests scalability with large numbers of channels.
    
    Rust equivalent: Many file.get_channel() calls or iteration
    """
    print("Benchmarking many channels read...")
    
    # Use existing test files that have many channels
    test_files_dir = Path(__file__).parent / "test_files"
    
    for tdms_file in test_files_dir.glob("*many*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = "stress_many_channels"
        
        # Open and analyze file structure
        with benchmark_context(
            "stress_many_channels",
            file_type,
            0,  # Will be updated
            0,  # Will be updated
            "mixed",
            "open_many_channels",
            file_size_mb,
            "Open file with many channels"
        ):
            tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
            
            # Count channels
            total_channels = 0
            total_samples = 0
            for group in tdms_file_obj.groups():
                for channel in group.channels():
                    total_channels += 1
                    if hasattr(channel, 'data') and channel.data is not None:
                        total_samples += len(channel.data)
        
        # Update result with actual counts
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.channels = total_channels
            last_result.samples = total_samples
        
        # Read all channels individually
        if total_channels > 0:
            with benchmark_context(
                "stress_many_channels",
                file_type,
                total_channels,
                total_samples,
                "mixed",
                "read_all_individual",
                file_size_mb,
                f"Read {total_channels} channels individually"
            ):
                channel_data = {}
                for group in tdms_file_obj.groups():
                    for channel in group.channels():
                        if hasattr(channel, 'data') and channel.data is not None:
                            channel_data[f"{group.name}/{channel.name}"] = channel[:]
                _ = len(channel_data)


def benchmark_very_large_channels():
    """
    Benchmark very large single channels.
    
    Tests memory usage and performance with massive data arrays.
    
    Rust equivalent: Large TdmsData variants
    """
    print("Benchmarking very large channels...")
    
    # Create and test very large channels
    large_sizes = [
        (1000000, "1M_samples"),
        (10000000, "10M_samples"),
    ]
    
    for samples, size_name in large_sizes:
        with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
            tmp_path = tmp_file.name
        
        try:
            # Create large file
            print(f"  Creating {size_name} test file...")
            with TdmsWriter(tmp_path) as tdms_writer:
                data = np.random.random(samples).astype(np.float64)
                channel = ChannelObject('Data', 'LargeChannel', data)
                channel.properties['Description'] = f'Large channel with {samples} samples'
                tdms_writer.write_data([channel])
            
            file_size_mb = get_file_size_mb(Path(tmp_path))
            
            # Benchmark reading the large channel
            with benchmark_context(
                "stress_large_channels",
                size_name,
                1,
                samples,
                "float64",
                "read_large_channel",
                file_size_mb,
                f"Read single channel with {samples} samples"
            ):
                tdms_file_obj = nptdms.TdmsFile.read(tmp_path)
                large_channel = None
                for group in tdms_file_obj.groups():
                    for channel in group.channels():
                        if hasattr(channel, 'data') and channel.data is not None:
                            large_channel = channel
                            break
                    if large_channel:
                        break
                
                if large_channel:
                    data = large_channel[:]
                    _ = len(data)
            
            # Benchmark chunked reading of large channel
            chunk_size = min(100000, samples // 10)
            num_chunks = min(10, samples // chunk_size)
            
            with benchmark_context(
                "stress_large_channels",
                size_name,
                1,
                chunk_size * num_chunks,
                "float64",
                "read_large_chunked",
                file_size_mb * (num_chunks * chunk_size / samples),
                f"Read {num_chunks} chunks of {chunk_size} samples"
            ):
                if large_channel:
                    chunks = []
                    for i in range(num_chunks):
                        start = i * chunk_size
                        end = start + chunk_size
                        chunk = large_channel[start:end]
                        chunks.append(chunk)
                    _ = len(chunks)
        
        finally:
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)


def benchmark_tiny_channels():
    """
    Benchmark files with many tiny channels.
    
    Tests overhead when channel data is very small.
    
    Rust equivalent: Many small TdmsData variants
    """
    print("Benchmarking tiny channels...")
    
    # Create file with many tiny channels
    num_tiny_channels = 1000
    samples_per_channel = 5  # Very small
    
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        # Create file with many tiny channels
        with benchmark_context(
            "stress_tiny_channels",
            "many_tiny",
            num_tiny_channels,
            num_tiny_channels * samples_per_channel,
            "float64",
            "write_tiny_channels",
            0,  # Will be calculated
            f"Write {num_tiny_channels} channels with {samples_per_channel} samples each"
        ):
            with TdmsWriter(tmp_path) as tdms_writer:
                channels = []
                for i in range(num_tiny_channels):
                    data = np.random.random(samples_per_channel)
                    channel = ChannelObject('Data', f'Tiny_{i:04d}', data)
                    channel.properties['Index'] = i
                    channels.append(channel)
                tdms_writer.write_data(channels)
        
        file_size_mb = get_file_size_mb(Path(tmp_path))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
        
        # Read all tiny channels
        with benchmark_context(
            "stress_tiny_channels",
            "many_tiny",
            num_tiny_channels,
            num_tiny_channels * samples_per_channel,
            "float64",
            "read_tiny_channels",
            file_size_mb,
            f"Read {num_tiny_channels} tiny channels"
        ):
            tdms_file_obj = nptdms.TdmsFile.read(tmp_path)
            tiny_data = {}
            for group in tdms_file_obj.groups():
                for channel in group.channels():
                    if hasattr(channel, 'data') and channel.data is not None:
                        tiny_data[channel.name] = channel[:]
            _ = len(tiny_data)
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def benchmark_property_heavy_files():
    """
    Benchmark files with extensive properties.
    
    Tests property handling overhead.
    
    Rust equivalent: Many add_property() calls
    """
    print("Benchmarking property-heavy files...")
    
    # Create file with many properties
    num_channels = 10
    properties_per_channel = 100
    samples_per_channel = 1000
    
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        # Create file with extensive properties
        with benchmark_context(
            "stress_properties",
            "property_heavy",
            num_channels,
            num_channels * samples_per_channel,
            "float64",
            "write_many_properties",
            0,  # Will be calculated
            f"Write {num_channels} channels with {properties_per_channel} properties each"
        ):
            with TdmsWriter(tmp_path) as tdms_writer:
                # Add extensive file properties
                tdms_writer.file_properties = {}
                for i in range(50):
                    tdms_writer.file_properties[f'File_Prop_{i:03d}'] = f'File_Value_{i}'
                    tdms_writer.file_properties[f'File_Num_{i:03d}'] = i * 1.5
                    tdms_writer.file_properties[f'File_Bool_{i:03d}'] = i % 2 == 0
                
                channels = []
                for ch in range(num_channels):
                    data = np.random.random(samples_per_channel)
                    channel = ChannelObject('Data', f'Channel_{ch:03d}', data)
                    
                    # Add many properties to each channel
                    for prop in range(properties_per_channel):
                        channel.properties[f'Prop_{prop:03d}'] = f'Value_{prop}'
                        channel.properties[f'Num_{prop:03d}'] = prop * 2.5
                        channel.properties[f'Bool_{prop:03d}'] = prop % 3 == 0
                    
                    channels.append(channel)
                
                tdms_writer.write_data(channels)
        
        file_size_mb = get_file_size_mb(Path(tmp_path))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
        
        # Read and access all properties
        with benchmark_context(
            "stress_properties",
            "property_heavy",
            num_channels,
            num_channels * samples_per_channel,
            "properties",
            "read_all_properties",
            file_size_mb,
            f"Read file and access all {num_channels * properties_per_channel} properties"
        ):
            tdms_file_obj = nptdms.TdmsFile.read(tmp_path)
            
            # Access file properties
            file_props = dict(tdms_file_obj.properties) if hasattr(tdms_file_obj, 'properties') else {}
            
            # Access all channel properties
            all_properties = {}
            for group in tdms_file_obj.groups():
                for channel in group.channels():
                    if hasattr(channel, 'properties'):
                        for key, value in channel.properties.items():
                            all_properties[f"{group.name}/{channel.name}/{key}"] = value
            
            _ = len(all_properties)
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def benchmark_mixed_data_type_stress():
    """
    Benchmark files with many different data types.
    
    Tests type handling overhead with complex mixed files.
    
    Rust equivalent: Many different TdmsData enum variants
    """
    print("Benchmarking mixed data type stress...")
    
    # Create file with many channels of different types
    channels_per_type = 20
    samples_per_channel = 2000
    
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        # Create mixed type file
        data_types = [
            ("float64", np.float64, lambda n: np.random.random(n)),
            ("float32", np.float32, lambda n: np.random.random(n).astype(np.float32)),
            ("int32", np.int32, lambda n: np.random.randint(-1000, 1000, n).astype(np.int32)),
            ("int16", np.int16, lambda n: np.random.randint(-100, 100, n).astype(np.int16)),
            ("uint8", np.uint8, lambda n: np.random.randint(0, 255, n).astype(np.uint8)),
        ]
        
        total_channels = len(data_types) * channels_per_type
        
        with benchmark_context(
            "stress_mixed_types",
            "mixed_stress",
            total_channels,
            total_channels * samples_per_channel,
            "mixed",
            "write_mixed_stress",
            0,  # Will be calculated
            f"Write {total_channels} channels with {len(data_types)} different data types"
        ):
            with TdmsWriter(tmp_path) as tdms_writer:
                channels = []
                
                for type_name, dtype, generator in data_types:
                    for i in range(channels_per_type):
                        data = generator(samples_per_channel)
                        channel = ChannelObject('Mixed', f'{type_name}_{i:03d}', data)
                        channel.properties['DataType'] = type_name
                        channel.properties['TypeIndex'] = i
                        channels.append(channel)
                
                tdms_writer.write_data(channels)
        
        file_size_mb = get_file_size_mb(Path(tmp_path))
        if hasattr(benchmark_context, '_results') and benchmark_context._results:
            last_result = benchmark_context._results[-1]
            last_result.mb_per_sec = file_size_mb / last_result.time_sec if last_result.time_sec > 0 else 0
        
        # Read all mixed type channels
        with benchmark_context(
            "stress_mixed_types",
            "mixed_stress",
            total_channels,
            total_channels * samples_per_channel,
            "mixed",
            "read_mixed_stress",
            file_size_mb,
            f"Read {total_channels} channels with mixed data types"
        ):
            tdms_file_obj = nptdms.TdmsFile.read(tmp_path)
            
            type_data = {}
            for group in tdms_file_obj.groups():
                for channel in group.channels():
                    if hasattr(channel, 'data') and channel.data is not None:
                        data_type = channel.properties.get('DataType', 'unknown')
                        if data_type not in type_data:
                            type_data[data_type] = []
                        type_data[data_type].append(channel[:])
            
            _ = len(type_data)
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def benchmark_pathological_access_patterns():
    """
    Benchmark pathological access patterns.
    
    Tests worst-case scenarios for data access.
    """
    print("Benchmarking pathological access patterns...")
    
    # Use existing test files
    test_files_dir = Path(__file__).parent / "test_files"
    ensure_test_files_exist()
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Skip very large files for pathological tests
        if file_size_mb > 20:
            continue
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find channels for testing
        test_channels = []
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None and len(channel.data) > 100:
                    test_channels.append(channel)
        
        if not test_channels:
            continue
        
        # Random sparse access pattern
        test_channel = test_channels[0]
        channel_len = len(test_channel.data)
        
        if channel_len > 1000:
            # Generate sparse random indices
            num_sparse = min(100, channel_len // 100)
            sparse_indices = sorted(np.random.choice(channel_len, size=num_sparse, replace=False))
            
            with benchmark_context(
                "stress_pathological",
                file_type,
                1,
                num_sparse,
                "sparse_access",
                "random_sparse",
                file_size_mb * (num_sparse / channel_len),
                f"Sparse random access: {num_sparse} indices from {channel_len} samples"
            ):
                sparse_data = []
                for idx in sparse_indices:
                    sparse_data.append(test_channel[idx])
                _ = len(sparse_data)
        
        # Reverse access pattern
        if channel_len > 1000:
            reverse_size = min(1000, channel_len)
            
            with benchmark_context(
                "stress_pathological",
                file_type,
                1,
                reverse_size,
                "reverse_access",
                "reverse_order",
                file_size_mb * (reverse_size / channel_len),
                f"Reverse order access: {reverse_size} samples backwards"
            ):
                reverse_data = []
                for i in range(reverse_size):
                    idx = channel_len - 1 - i
                    reverse_data.append(test_channel[idx])
                _ = len(reverse_data)


def main():
    """Run all stress benchmarks."""
    print("Starting TDMS Stress Benchmarks")
    print("=" * 50)
    
    # Ensure test files exist
    ensure_test_files_exist()
    
    clear_benchmark_results()
    
    # Run all stress test categories
    benchmark_many_channels_read()
    benchmark_very_large_channels()
    benchmark_tiny_channels()
    benchmark_property_heavy_files()
    benchmark_mixed_data_type_stress()
    benchmark_pathological_access_patterns()
    
    # Save results
    results = get_benchmark_results()
    output_dir = Path(__file__).parent / "results"
    write_csv_results(results, output_dir / "stress_benchmarks.csv")
    
    print(f"\nCompleted {len(results)} stress benchmarks")
    print(f"Results saved to: {output_dir / 'stress_benchmarks.csv'}")


if __name__ == "__main__":
    main()