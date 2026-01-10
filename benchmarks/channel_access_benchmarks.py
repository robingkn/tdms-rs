"""
TDMS Channel Access Pattern Benchmarks

Benchmarks focused on different channel access patterns and the overhead
of Python abstractions. These tests are critical for understanding the
performance characteristics that Rust can improve upon.

Rust Equivalent Operations:
- file['group']['channel'] -> file.get_channel('group', 'channel')
- channel[:] -> channel.as_f64() and similar type accessors
- channel.properties -> channel.properties (direct access)
- len(channel) -> channel.data_len()
"""

import numpy as np
from pathlib import Path
import time
from typing import List, Dict, Any

try:
    import nptdms
except ImportError:
    print("Error: nptdms not installed. Run: pip install nptdms")
    exit(1)

from benchmark_utils import (
    benchmark_context, get_benchmark_results, clear_benchmark_results,
    write_csv_results, get_file_size_mb, ensure_test_files_exist
)


def benchmark_channel_lookup_patterns(test_files_dir: Path):
    """
    Benchmark different channel lookup patterns.
    
    Tests:
    - Direct indexing: file['group']['channel']
    - Iteration-based lookup
    - Repeated lookups of the same channel
    
    Rust equivalent: file.get_channel() vs iteration
    """
    print("Benchmarking channel lookup patterns...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Skip very large files for lookup tests
        if file_size_mb > 100:
            continue
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find available groups and channels
        available_channels = []
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None:
                    available_channels.append((group.name, channel.name))
        
        if not available_channels:
            continue
        
        # Test direct indexing lookup
        group_name, channel_name = available_channels[0]
        
        with benchmark_context(
            "channel_access_lookup",
            file_type,
            1,
            0,  # Not applicable for lookup
            "lookup",
            "direct_index",
            0,  # Not applicable
            f"Direct lookup: file['{group_name}']['{channel_name}']"
        ):
            for _ in range(100):  # Repeat to measure lookup overhead
                channel = tdms_file_obj[group_name][channel_name]
                _ = channel.name  # Force access
        
        # Test iteration-based lookup
        with benchmark_context(
            "channel_access_lookup",
            file_type,
            len(available_channels),
            0,
            "lookup",
            "iteration_search",
            0,
            "Find channel by iterating through all channels"
        ):
            target_group, target_channel = available_channels[0]
            found_channel = None
            
            for group in tdms_file_obj.groups():
                if group.name == target_group:
                    for channel in group.channels():
                        if channel.name == target_channel:
                            found_channel = channel
                            break
                    break
            
            _ = found_channel.name if found_channel else None
        
        # Test repeated lookups (caching behavior)
        with benchmark_context(
            "channel_access_lookup",
            file_type,
            1,
            0,
            "lookup",
            "repeated_lookup",
            0,
            "Repeated lookups of same channel (100x)"
        ):
            for _ in range(100):
                channel = tdms_file_obj[group_name][channel_name]
                _ = channel.name


def benchmark_data_access_patterns(test_files_dir: Path):
    """
    Benchmark different data access patterns.
    
    Tests:
    - Full data access: channel[:]
    - Chunked access: channel[i:i+chunk_size]
    - Single element access: channel[i]
    - Property access overhead
    
    Rust equivalent: channel.as_f64() vs slicing operations
    """
    print("Benchmarking data access patterns...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Skip very large files for detailed access tests
        if file_size_mb > 50:
            continue
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find a suitable test channel
        test_channel = None
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None and len(channel.data) > 1000:
                    test_channel = channel
                    break
            if test_channel:
                break
        
        if not test_channel:
            continue
        
        channel_len = len(test_channel.data)
        data_type = type(test_channel.data[0]).__name__ if channel_len > 0 else "unknown"
        
        # Full data access
        with benchmark_context(
            "channel_access_data",
            file_type,
            1,
            channel_len,
            data_type,
            "full_access",
            file_size_mb,
            "Full channel data access: channel[:]"
        ):
            full_data = test_channel[:]
            _ = len(full_data)
        
        # Chunked access (if channel is large enough)
        if channel_len > 10000:
            chunk_size = 1000
            num_chunks = min(10, channel_len // chunk_size)
            
            with benchmark_context(
                "channel_access_data",
                file_type,
                1,
                chunk_size * num_chunks,
                data_type,
                "chunked_access",
                file_size_mb * (num_chunks * chunk_size / channel_len),
                f"Chunked access: {num_chunks} chunks of {chunk_size} samples"
            ):
                chunks = []
                for i in range(num_chunks):
                    start = i * chunk_size
                    end = start + chunk_size
                    chunk = test_channel[start:end]
                    chunks.append(chunk)
                _ = len(chunks)
        
        # Single element access (measure indexing overhead)
        if channel_len > 100:
            num_elements = min(100, channel_len)
            indices = np.random.choice(channel_len, size=num_elements, replace=False)
            
            with benchmark_context(
                "channel_access_data",
                file_type,
                1,
                num_elements,
                data_type,
                "single_element",
                0,  # Negligible data size
                f"Single element access: {num_elements} random indices"
            ):
                elements = []
                for idx in indices:
                    element = test_channel[idx]
                    elements.append(element)
                _ = len(elements)


def benchmark_property_access_overhead(test_files_dir: Path):
    """
    Benchmark property access overhead.
    
    Tests:
    - Property dictionary access
    - Property iteration
    - Property type conversion
    
    Rust equivalent: channel.properties access and iteration
    """
    print("Benchmarking property access overhead...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find channels with properties
        channels_with_props = []
        for group in tdms_file_obj.groups():
            for channel in group.channels():
                if hasattr(channel, 'properties') and len(channel.properties) > 0:
                    channels_with_props.append(channel)
        
        if not channels_with_props:
            continue
        
        test_channel = channels_with_props[0]
        num_properties = len(test_channel.properties)
        
        # Property access by key
        property_keys = list(test_channel.properties.keys())
        if property_keys:
            first_key = property_keys[0]
            
            with benchmark_context(
                "channel_access_properties",
                file_type,
                1,
                num_properties,
                "property",
                "key_access",
                0,
                f"Property access by key: {first_key}"
            ):
                for _ in range(100):  # Repeat to measure overhead
                    value = test_channel.properties.get(first_key)
                    _ = str(value)  # Force string conversion
        
        # Property iteration
        with benchmark_context(
            "channel_access_properties",
            file_type,
            1,
            num_properties,
            "property",
            "iteration",
            0,
            f"Iterate through {num_properties} properties"
        ):
            for _ in range(10):  # Repeat iteration
                prop_list = []
                for key, value in test_channel.properties.items():
                    prop_list.append((key, str(value)))
                _ = len(prop_list)
        
        # Property keys access
        with benchmark_context(
            "channel_access_properties",
            file_type,
            1,
            num_properties,
            "property",
            "keys_access",
            0,
            "Access all property keys"
        ):
            for _ in range(100):
                keys = list(test_channel.properties.keys())
                _ = len(keys)


def benchmark_metadata_only_operations(test_files_dir: Path):
    """
    Benchmark metadata-only operations (no data access).
    
    Tests operations that only touch metadata:
    - Channel enumeration
    - Property inspection
    - Structure traversal
    
    Rust equivalent: Metadata access without data loading
    """
    print("Benchmarking metadata-only operations...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Count total structure
        total_groups = len(list(tdms_file_obj.groups()))
        total_channels = sum(len(list(group.channels())) for group in tdms_file_obj.groups())
        
        # Enumerate all groups and channels (metadata only)
        with benchmark_context(
            "channel_access_metadata",
            file_type,
            total_channels,
            0,  # No data accessed
            "metadata",
            "enumerate_structure",
            0,
            f"Enumerate {total_groups} groups, {total_channels} channels"
        ):
            structure = {}
            for group in tdms_file_obj.groups():
                group_channels = []
                for channel in group.channels():
                    # Only access metadata, not data
                    channel_info = {
                        'name': channel.name,
                        'has_data': hasattr(channel, 'data') and channel.data is not None,
                        'data_len': len(channel.data) if hasattr(channel, 'data') and channel.data is not None else 0,
                        'num_properties': len(channel.properties) if hasattr(channel, 'properties') else 0
                    }
                    group_channels.append(channel_info)
                structure[group.name] = group_channels
            _ = len(structure)
        
        # Property summary (metadata only)
        with benchmark_context(
            "channel_access_metadata",
            file_type,
            total_channels,
            0,
            "metadata",
            "property_summary",
            0,
            "Summarize all properties without accessing data"
        ):
            property_summary = {}
            for group in tdms_file_obj.groups():
                for channel in group.channels():
                    if hasattr(channel, 'properties'):
                        for key in channel.properties.keys():
                            if key not in property_summary:
                                property_summary[key] = 0
                            property_summary[key] += 1
            _ = len(property_summary)


def benchmark_repeated_channel_access(test_files_dir: Path):
    """
    Benchmark repeated access to the same channel data.
    
    Tests caching behavior and repeated access patterns.
    
    Rust equivalent: Multiple calls to the same channel accessor
    """
    print("Benchmarking repeated channel access...")
    
    for tdms_file in test_files_dir.glob("*.tdms"):
        file_size_mb = get_file_size_mb(tdms_file)
        file_type = tdms_file.stem.split('_')[0]
        
        # Skip very large files
        if file_size_mb > 20:
            continue
        
        tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
        
        # Find a suitable test channel
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
        
        # Repeated full access
        num_repeats = 10
        with benchmark_context(
            "channel_access_repeated",
            file_type,
            1,
            channel_len * num_repeats,
            data_type,
            "repeated_full",
            file_size_mb * num_repeats,
            f"Repeated full access ({num_repeats}x)"
        ):
            for _ in range(num_repeats):
                data = test_channel[:]
                _ = len(data)
        
        # Repeated slice access
        if channel_len > 1000:
            slice_size = min(100, channel_len // 10)
            with benchmark_context(
                "channel_access_repeated",
                file_type,
                1,
                slice_size * num_repeats,
                data_type,
                "repeated_slice",
                file_size_mb * (slice_size / channel_len) * num_repeats,
                f"Repeated slice access ({num_repeats}x, {slice_size} samples)"
            ):
                for _ in range(num_repeats):
                    data = test_channel[:slice_size]
                    _ = len(data)


def main():
    """Run all channel access benchmarks."""
    print("Starting TDMS Channel Access Benchmarks")
    print("=" * 50)
    
    # Ensure test files exist
    ensure_test_files_exist()
    
    test_files_dir = Path(__file__).parent / "test_files"
    clear_benchmark_results()
    
    # Run all benchmark categories
    benchmark_channel_lookup_patterns(test_files_dir)
    benchmark_data_access_patterns(test_files_dir)
    benchmark_property_access_overhead(test_files_dir)
    benchmark_metadata_only_operations(test_files_dir)
    benchmark_repeated_channel_access(test_files_dir)
    
    # Save results
    results = get_benchmark_results()
    output_dir = Path(__file__).parent / "results"
    write_csv_results(results, output_dir / "channel_access_benchmarks.csv")
    
    print(f"\nCompleted {len(results)} channel access benchmarks")
    print(f"Results saved to: {output_dir / 'channel_access_benchmarks.csv'}")


if __name__ == "__main__":
    main()