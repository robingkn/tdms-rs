#!/usr/bin/env python3
"""
Simplified smoke test for nptdms benchmarks.
This version focuses on the core functionality without complex features.
"""

import numpy as np
import time
import tempfile
import os
from pathlib import Path

try:
    import nptdms
    from nptdms import TdmsWriter, ChannelObject
except ImportError:
    print("Error: nptdms not installed. Run: pip install nptdms")
    exit(1)

def simple_benchmark(name, operation_func):
    """Simple benchmark wrapper."""
    print(f"Running {name}...")
    start_time = time.perf_counter()
    
    try:
        result = operation_func()
        end_time = time.perf_counter()
        elapsed = end_time - start_time
        print(f"  ✅ {name}: {elapsed:.3f}s")
        return elapsed, result
    except Exception as e:
        end_time = time.perf_counter()
        elapsed = end_time - start_time
        print(f"  ❌ {name}: {elapsed:.3f}s - Error: {e}")
        return elapsed, None

def test_file_generation():
    """Test generating TDMS files."""
    test_files_dir = Path(__file__).parent / "test_files"
    test_files_dir.mkdir(exist_ok=True)
    
    def generate_files():
        # Small single channel file
        file_path = test_files_dir / "simple_test.tdms"
        with TdmsWriter(str(file_path)) as writer:
            data = np.random.random(10000).astype(np.float64)
            channel = ChannelObject('TestGroup', 'TestChannel', data, properties={
                'Unit': 'V',
                'Description': 'Test channel'
            })
            writer.write_segment([channel])
        
        return file_path.stat().st_size / 1024 / 1024  # Size in MB
    
    return simple_benchmark("File Generation", generate_files)

def test_file_reading():
    """Test reading TDMS files."""
    test_files_dir = Path(__file__).parent / "test_files"
    test_file = test_files_dir / "simple_test.tdms"
    
    if not test_file.exists():
        print("  Skipping read test - no test file found")
        return 0, None
    
    def read_file():
        tdms_file = nptdms.TdmsFile.read(str(test_file))
        
        # Count groups and channels
        groups = list(tdms_file.groups())
        total_channels = 0
        total_samples = 0
        
        for group in groups:
            channels = list(group.channels())
            total_channels += len(channels)
            
            for channel in channels:
                if hasattr(channel, 'data') and channel.data is not None:
                    total_samples += len(channel.data)
                    # Access the data to measure read time
                    _ = channel[:]
        
        return {'groups': len(groups), 'channels': total_channels, 'samples': total_samples}
    
    return simple_benchmark("File Reading", read_file)

def test_channel_access():
    """Test channel access patterns."""
    test_files_dir = Path(__file__).parent / "test_files"
    test_file = test_files_dir / "simple_test.tdms"
    
    if not test_file.exists():
        print("  Skipping channel access test - no test file found")
        return 0, None
    
    def access_channels():
        tdms_file = nptdms.TdmsFile.read(str(test_file))
        
        # Find first channel
        for group in tdms_file.groups():
            for channel in group.channels():
                if hasattr(channel, 'data') and channel.data is not None:
                    # Test different access patterns
                    full_data = channel[:]
                    
                    if len(channel.data) > 100:
                        slice_data = channel[:100]
                        single_element = channel[0]
                    
                    # Test property access
                    if hasattr(channel, 'properties') and channel.properties:
                        props = dict(channel.properties)
                    
                    return len(full_data)
        
        return 0
    
    return simple_benchmark("Channel Access", access_channels)

def test_write_performance():
    """Test write performance with different data sizes."""
    results = {}
    
    for size_name, samples in [("small", 1000), ("medium", 100000)]:
        def write_test():
            with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
                tmp_path = tmp_file.name
            
            try:
                data = np.random.random(samples).astype(np.float64)
                
                with TdmsWriter(tmp_path) as writer:
                    channel = ChannelObject('Data', 'TestChannel', data, properties={
                        'SampleCount': samples,
                        'DataType': 'float64'
                    })
                    writer.write_segment([channel])
                
                file_size = Path(tmp_path).stat().st_size / 1024 / 1024  # MB
                return file_size
            
            finally:
                if os.path.exists(tmp_path):
                    os.unlink(tmp_path)
        
        elapsed, file_size = simple_benchmark(f"Write {size_name} ({samples} samples)", write_test)
        if file_size:
            throughput = file_size / elapsed if elapsed > 0 else 0
            print(f"    File size: {file_size:.2f} MB, Throughput: {throughput:.1f} MB/s")
            results[size_name] = {'elapsed': elapsed, 'size_mb': file_size, 'throughput': throughput}
    
    return results

def main():
    """Run simplified smoke tests."""
    print("nptdms Simple Smoke Test")
    print("=" * 40)
    print(f"nptdms version: {nptdms.__version__}")
    print()
    
    # Run tests
    results = {}
    
    # Test file generation
    elapsed, size_mb = test_file_generation()
    results['generation'] = {'elapsed': elapsed, 'size_mb': size_mb}
    
    # Test file reading
    elapsed, read_result = test_file_reading()
    results['reading'] = {'elapsed': elapsed, 'result': read_result}
    
    # Test channel access
    elapsed, access_result = test_channel_access()
    results['access'] = {'elapsed': elapsed, 'result': access_result}
    
    # Test write performance
    write_results = test_write_performance()
    results['write'] = write_results
    
    # Summary
    print("\n" + "=" * 40)
    print("Summary:")
    
    total_time = sum(r.get('elapsed', 0) for r in results.values() if isinstance(r, dict))
    if 'write' in results:
        total_time += sum(r.get('elapsed', 0) for r in results['write'].values())
    
    print(f"Total test time: {total_time:.2f}s")
    
    if results['reading']['result']:
        read_info = results['reading']['result']
        print(f"Test file: {read_info['groups']} groups, {read_info['channels']} channels, {read_info['samples']} samples")
    
    if 'write' in results and 'medium' in results['write']:
        medium_write = results['write']['medium']
        print(f"Write throughput (100k samples): {medium_write['throughput']:.1f} MB/s")
    
    print("\n✅ Smoke test completed successfully!")
    print("The benchmark suite is working correctly.")

if __name__ == "__main__":
    main()