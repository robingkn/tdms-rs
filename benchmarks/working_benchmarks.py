#!/usr/bin/env python3
"""
Working nptdms benchmarks with proper API usage.
This version uses the correct nptdms API and includes automatic cleanup.
"""

import numpy as np
import time
import tempfile
import os
import csv
from pathlib import Path
import psutil
import tracemalloc

try:
    import nptdms
    from nptdms import TdmsWriter, ChannelObject
except ImportError:
    print("Error: nptdms not installed. Run: pip install nptdms")
    exit(1)

class BenchmarkResult:
    def __init__(self, name, operation, time_sec, file_size_mb=0, throughput_mb_s=0, notes=""):
        self.name = name
        self.operation = operation
        self.time_sec = time_sec
        self.file_size_mb = file_size_mb
        self.throughput_mb_s = throughput_mb_s
        self.notes = notes

def benchmark_operation(name, operation_func, cleanup_func=None):
    """Benchmark an operation with timing and memory tracking."""
    print(f"  Running {name}...")
    
    # Start memory tracking
    tracemalloc.start()
    process = psutil.Process()
    start_memory = process.memory_info().rss / 1024 / 1024
    
    start_time = time.perf_counter()
    
    try:
        result = operation_func()
        end_time = time.perf_counter()
        
        # Calculate metrics
        elapsed = end_time - start_time
        current_memory = process.memory_info().rss / 1024 / 1024
        peak_memory = max(current_memory - start_memory, 0)
        
        tracemalloc.stop()
        
        print(f"    ✅ {elapsed:.3f}s, Memory: {peak_memory:.1f}MB")
        
        return elapsed, result, peak_memory
        
    except Exception as e:
        end_time = time.perf_counter()
        elapsed = end_time - start_time
        tracemalloc.stop()
        
        print(f"    ❌ {elapsed:.3f}s - Error: {e}")
        return elapsed, None, 0
    
    finally:
        if cleanup_func:
            try:
                cleanup_func()
            except:
                pass

def generate_test_files():
    """Generate test files for benchmarking."""
    print("Generating test files...")
    
    test_files_dir = Path(__file__).parent / "test_files"
    test_files_dir.mkdir(exist_ok=True)
    
    generated_files = []
    
    # Small single channel
    def gen_small_single():
        file_path = test_files_dir / "bench_small_single.tdms"
        with TdmsWriter(str(file_path)) as writer:
            data = np.random.random(10000).astype(np.float64)
            channel = ChannelObject('Data', 'Channel1', data, properties={
                'Unit': 'V',
                'Description': 'Small single channel'
            })
            writer.write_segment([channel])
        return file_path
    
    elapsed, file_path, _ = benchmark_operation("Small single channel", gen_small_single)
    if file_path:
        size_mb = file_path.stat().st_size / 1024 / 1024
        generated_files.append(('small_single', file_path, size_mb))
    
    # Small multi-channel
    def gen_small_multi():
        file_path = test_files_dir / "bench_small_multi.tdms"
        with TdmsWriter(str(file_path)) as writer:
            channels = []
            for i in range(5):
                data = np.random.random(5000).astype(np.float64)
                channel = ChannelObject('Sensors', f'Channel_{i}', data, properties={
                    'Index': i,
                    'Unit': 'V'
                })
                channels.append(channel)
            writer.write_segment(channels)
        return file_path
    
    elapsed, file_path, _ = benchmark_operation("Small multi-channel", gen_small_multi)
    if file_path:
        size_mb = file_path.stat().st_size / 1024 / 1024
        generated_files.append(('small_multi', file_path, size_mb))
    
    # Medium file (only if not in smoke mode)
    def gen_medium():
        file_path = test_files_dir / "bench_medium.tdms"
        with TdmsWriter(str(file_path)) as writer:
            data = np.random.random(1000000).astype(np.float64)  # 1M samples
            channel = ChannelObject('Data', 'LargeChannel', data, properties={
                'Description': 'Medium sized channel for throughput testing'
            })
            writer.write_segment([channel])
        return file_path
    
    elapsed, file_path, _ = benchmark_operation("Medium file", gen_medium)
    if file_path:
        size_mb = file_path.stat().st_size / 1024 / 1024
        generated_files.append(('medium', file_path, size_mb))
    
    print(f"Generated {len(generated_files)} test files:")
    for name, path, size in generated_files:
        print(f"  {name}: {path.name} ({size:.1f} MB)")
    
    return generated_files

def benchmark_read_operations(test_files):
    """Benchmark read operations."""
    print("\nBenchmarking read operations...")
    results = []
    
    for file_type, file_path, file_size_mb in test_files:
        print(f"\n  Testing {file_type} ({file_size_mb:.1f} MB):")
        
        # File opening
        def open_file():
            return nptdms.TdmsFile.read(str(file_path))
        
        elapsed, tdms_file, memory = benchmark_operation("File opening", open_file)
        results.append(BenchmarkResult(
            f"read_{file_type}", "open_file", elapsed, file_size_mb, 
            file_size_mb / elapsed if elapsed > 0 else 0,
            f"Open and parse {file_type} file"
        ))
        
        if tdms_file:
            # Count structure
            groups = list(tdms_file.groups())
            total_channels = sum(len(list(group.channels())) for group in groups)
            
            # Channel access
            def access_channels():
                total_samples = 0
                for group in groups:
                    for channel in group.channels():
                        if hasattr(channel, 'data') and channel.data is not None:
                            data = channel[:]
                            total_samples += len(data)
                return total_samples
            
            elapsed, total_samples, memory = benchmark_operation("Channel data access", access_channels)
            results.append(BenchmarkResult(
                f"read_{file_type}", "channel_access", elapsed, file_size_mb,
                file_size_mb / elapsed if elapsed > 0 else 0,
                f"Access {total_channels} channels, {total_samples} samples"
            ))
            
            # Property access (if channels have properties)
            def access_properties():
                prop_count = 0
                for group in groups:
                    for channel in group.channels():
                        if hasattr(channel, 'properties') and channel.properties:
                            for key, value in channel.properties.items():
                                prop_count += 1
                return prop_count
            
            elapsed, prop_count, memory = benchmark_operation("Property access", access_properties)
            results.append(BenchmarkResult(
                f"read_{file_type}", "property_access", elapsed, 0,
                0, f"Access {prop_count} properties"
            ))
    
    return results

def benchmark_write_operations():
    """Benchmark write operations."""
    print("\nBenchmarking write operations...")
    results = []
    
    # Test different data sizes and types
    test_configs = [
        ("small", 1000, np.float64),
        ("medium", 100000, np.float64),
        ("large", 1000000, np.float64),
        ("int32", 100000, np.int32),
        ("float32", 100000, np.float32),
    ]
    
    for config_name, samples, dtype in test_configs:
        print(f"\n  Testing {config_name} ({samples} samples, {dtype.__name__}):")
        
        def write_test():
            with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
                tmp_path = tmp_file.name
            
            try:
                if dtype == np.float64 or dtype == np.float32:
                    data = np.random.random(samples).astype(dtype)
                else:
                    data = np.random.randint(0, 1000, samples).astype(dtype)
                
                with TdmsWriter(tmp_path) as writer:
                    channel = ChannelObject('Data', 'TestChannel', data, properties={
                        'SampleCount': samples,
                        'DataType': dtype.__name__
                    })
                    writer.write_segment([channel])
                
                file_size = Path(tmp_path).stat().st_size / 1024 / 1024
                return tmp_path, file_size
            
            except Exception as e:
                if os.path.exists(tmp_path):
                    os.unlink(tmp_path)
                raise e
        
        def cleanup_write(paths):
            if isinstance(paths, tuple):
                tmp_path, _ = paths
                if os.path.exists(tmp_path):
                    os.unlink(tmp_path)
        
        elapsed, result, memory = benchmark_operation(f"Write {config_name}", write_test, 
                                                     lambda: cleanup_write(result) if result else None)
        
        if result:
            tmp_path, file_size_mb = result
            throughput = file_size_mb / elapsed if elapsed > 0 else 0
            print(f"    File size: {file_size_mb:.2f} MB, Throughput: {throughput:.1f} MB/s")
            
            results.append(BenchmarkResult(
                f"write_{config_name}", "single_channel", elapsed, file_size_mb, throughput,
                f"Write {samples} {dtype.__name__} samples"
            ))
            
            # Clean up
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)
    
    return results

def benchmark_multi_channel_operations():
    """Benchmark multi-channel operations."""
    print("\nBenchmarking multi-channel operations...")
    results = []
    
    # Test different channel counts
    channel_configs = [
        ("few_channels", 5, 5000),
        ("many_channels", 20, 2000),
        ("very_many_channels", 50, 1000),
    ]
    
    for config_name, num_channels, samples_per_channel in channel_configs:
        print(f"\n  Testing {config_name} ({num_channels} channels, {samples_per_channel} samples each):")
        
        def multi_write_test():
            with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
                tmp_path = tmp_file.name
            
            try:
                with TdmsWriter(tmp_path) as writer:
                    channels = []
                    for i in range(num_channels):
                        data = np.random.random(samples_per_channel).astype(np.float64)
                        channel = ChannelObject('Data', f'Channel_{i:03d}', data, properties={
                            'ChannelIndex': i,
                            'Unit': 'V'
                        })
                        channels.append(channel)
                    
                    writer.write_segment(channels)
                
                file_size = Path(tmp_path).stat().st_size / 1024 / 1024
                return tmp_path, file_size
            
            except Exception as e:
                if os.path.exists(tmp_path):
                    os.unlink(tmp_path)
                raise e
        
        elapsed, result, memory = benchmark_operation(f"Multi-write {config_name}", multi_write_test)
        
        if result:
            tmp_path, file_size_mb = result
            throughput = file_size_mb / elapsed if elapsed > 0 else 0
            total_samples = num_channels * samples_per_channel
            
            results.append(BenchmarkResult(
                f"multi_write_{config_name}", "multi_channel", elapsed, file_size_mb, throughput,
                f"Write {num_channels} channels, {total_samples} total samples"
            ))
            
            # Test reading the multi-channel file
            def multi_read_test():
                tdms_file = nptdms.TdmsFile.read(tmp_path)
                total_read_samples = 0
                
                for group in tdms_file.groups():
                    for channel in group.channels():
                        if hasattr(channel, 'data') and channel.data is not None:
                            data = channel[:]
                            total_read_samples += len(data)
                
                return total_read_samples
            
            elapsed, read_samples, memory = benchmark_operation(f"Multi-read {config_name}", multi_read_test)
            
            results.append(BenchmarkResult(
                f"multi_read_{config_name}", "multi_channel", elapsed, file_size_mb,
                file_size_mb / elapsed if elapsed > 0 else 0,
                f"Read {num_channels} channels, {read_samples} total samples"
            ))
            
            # Clean up
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)
    
    return results

def save_results(results, output_file):
    """Save benchmark results to CSV."""
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['benchmark_name', 'operation', 'time_sec', 'file_size_mb', 'throughput_mb_s', 'notes'])
        
        for result in results:
            writer.writerow([
                result.name, result.operation, f"{result.time_sec:.6f}",
                f"{result.file_size_mb:.3f}", f"{result.throughput_mb_s:.3f}", result.notes
            ])

def cleanup_large_files(test_files_dir, size_threshold_mb=50):
    """Clean up large test files."""
    if not test_files_dir.exists():
        return
    
    large_files = []
    for file_path in test_files_dir.glob("*.tdms"):
        size_mb = file_path.stat().st_size / 1024 / 1024
        if size_mb > size_threshold_mb:
            large_files.append((file_path, size_mb))
    
    if large_files:
        print(f"\nCleaning up {len(large_files)} large files (>{size_threshold_mb} MB):")
        for file_path, size_mb in large_files:
            print(f"  Removing {file_path.name}: {size_mb:.1f} MB")
            try:
                file_path.unlink()
            except Exception as e:
                print(f"    Warning: Could not remove {file_path.name}: {e}")

def main():
    """Run comprehensive nptdms benchmarks."""
    print("nptdms Comprehensive Benchmarks")
    print("=" * 50)
    print(f"nptdms version: {nptdms.__version__}")
    print(f"numpy version: {np.__version__}")
    print()
    
    start_time = time.time()
    all_results = []
    
    try:
        # Generate test files
        test_files = generate_test_files()
        
        # Run benchmarks
        all_results.extend(benchmark_read_operations(test_files))
        all_results.extend(benchmark_write_operations())
        all_results.extend(benchmark_multi_channel_operations())
        
        # Calculate summary
        total_time = time.time() - start_time
        
        print(f"\n" + "=" * 50)
        print("Benchmark Summary:")
        print(f"Total tests: {len(all_results)}")
        print(f"Total time: {total_time:.1f}s")
        
        # Group results by category
        categories = {}
        for result in all_results:
            category = result.name.split('_')[0]
            if category not in categories:
                categories[category] = []
            categories[category].append(result)
        
        for category, results in categories.items():
            avg_time = sum(r.time_sec for r in results) / len(results)
            throughputs = [r.throughput_mb_s for r in results if r.throughput_mb_s > 0]
            avg_throughput = sum(throughputs) / len(throughputs) if throughputs else 0
            
            print(f"\n{category.upper()}:")
            print(f"  Tests: {len(results)}")
            print(f"  Avg time: {avg_time:.3f}s")
            if avg_throughput > 0:
                print(f"  Avg throughput: {avg_throughput:.1f} MB/s")
        
        # Save results
        results_dir = Path(__file__).parent / "results"
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        output_file = results_dir / f"nptdms_benchmarks_{timestamp}.csv"
        
        save_results(all_results, output_file)
        print(f"\nResults saved to: {output_file}")
        
        print("\n✅ Benchmark suite completed successfully!")
        
    finally:
        # Clean up large files
        test_files_dir = Path(__file__).parent / "test_files"
        cleanup_large_files(test_files_dir, size_threshold_mb=10)  # Clean files > 10MB

if __name__ == "__main__":
    main()