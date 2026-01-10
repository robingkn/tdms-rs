"""
Shared utilities for nptdms benchmarks.

This module provides common functionality for all benchmark scripts including:
- Timing and memory measurement
- CSV output formatting
- Test file management
- Result aggregation
"""

import csv
import json
import os
import time
import tracemalloc
from contextlib import contextmanager
from pathlib import Path
from typing import Dict, List, Optional, Any, Iterator
import psutil


class BenchmarkResult:
    """Container for benchmark results with consistent schema."""
    
    def __init__(
        self,
        benchmark_name: str,
        file_type: str,
        channels: int,
        samples: int,
        data_type: str,
        operation: str,
        time_sec: float,
        mb_per_sec: float,
        peak_memory_mb: float,
        notes: str = ""
    ):
        self.benchmark_name = benchmark_name
        self.file_type = file_type
        self.channels = channels
        self.samples = samples
        self.data_type = data_type
        self.operation = operation
        self.time_sec = time_sec
        self.mb_per_sec = mb_per_sec
        self.peak_memory_mb = peak_memory_mb
        self.notes = notes
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for CSV/JSON output."""
        return {
            'benchmark_name': self.benchmark_name,
            'file_type': self.file_type,
            'channels': self.channels,
            'samples': self.samples,
            'data_type': self.data_type,
            'operation': self.operation,
            'time_sec': self.time_sec,
            'mb_per_sec': self.mb_per_sec,
            'peak_memory_mb': self.peak_memory_mb,
            'notes': self.notes
        }


class BenchmarkTimer:
    """High-precision timer with memory tracking."""
    
    def __init__(self):
        self.start_time = None
        self.end_time = None
        self.peak_memory_mb = 0.0
        self.process = psutil.Process()
    
    def start(self):
        """Start timing and memory tracking."""
        # Force garbage collection for consistent memory baseline
        import gc
        gc.collect()
        
        # Start memory tracking
        tracemalloc.start()
        self.start_memory = self.process.memory_info().rss / 1024 / 1024
        
        # Start timing
        self.start_time = time.perf_counter()
    
    def stop(self) -> tuple[float, float]:
        """Stop timing and return (elapsed_time, peak_memory_mb)."""
        self.end_time = time.perf_counter()
        
        # Get peak memory usage
        current_memory = self.process.memory_info().rss / 1024 / 1024
        self.peak_memory_mb = max(current_memory - self.start_memory, 0.0)
        
        # Stop memory tracking
        tracemalloc.stop()
        
        elapsed = self.end_time - self.start_time
        return elapsed, self.peak_memory_mb


@contextmanager
def benchmark_context(
    benchmark_name: str,
    file_type: str,
    channels: int,
    samples: int,
    data_type: str,
    operation: str,
    file_size_mb: float = 0.0,
    notes: str = ""
) -> Iterator[BenchmarkResult]:
    """
    Context manager for consistent benchmark measurement.
    
    Usage:
        with benchmark_context("read_test", "small", 10, 1000, "f64", "full_read") as result:
            # Perform operation
            data = channel[:]
    """
    timer = BenchmarkTimer()
    timer.start()
    
    try:
        yield None  # Allow benchmark code to run
    finally:
        elapsed, peak_memory = timer.stop()
        
        # Calculate throughput
        mb_per_sec = file_size_mb / elapsed if elapsed > 0 and file_size_mb > 0 else 0.0
        
        # Create result object
        result = BenchmarkResult(
            benchmark_name=benchmark_name,
            file_type=file_type,
            channels=channels,
            samples=samples,
            data_type=data_type,
            operation=operation,
            time_sec=elapsed,
            mb_per_sec=mb_per_sec,
            peak_memory_mb=peak_memory,
            notes=notes
        )
        
        # Store result for collection
        if not hasattr(benchmark_context, '_results'):
            benchmark_context._results = []
        benchmark_context._results.append(result)


def get_benchmark_results() -> List[BenchmarkResult]:
    """Get all results from benchmark_context calls."""
    return getattr(benchmark_context, '_results', [])


def clear_benchmark_results():
    """Clear accumulated benchmark results."""
    benchmark_context._results = []


def write_csv_results(results: List[BenchmarkResult], output_file: Path):
    """Write benchmark results to CSV file."""
    if not results:
        return
    
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=[
            'benchmark_name', 'file_type', 'channels', 'samples', 'data_type',
            'operation', 'time_sec', 'mb_per_sec', 'peak_memory_mb', 'notes'
        ])
        writer.writeheader()
        for result in results:
            writer.writerow(result.to_dict())


def write_json_results(results: List[BenchmarkResult], output_file: Path):
    """Write benchmark results to JSON file."""
    if not results:
        return
    
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    data = {
        'timestamp': time.time(),
        'results': [result.to_dict() for result in results]
    }
    
    with open(output_file, 'w') as f:
        json.dump(data, f, indent=2)


def get_file_size_mb(file_path: Path) -> float:
    """Get file size in MB."""
    return file_path.stat().st_size / 1024 / 1024


def ensure_test_files_exist():
    """Ensure benchmark test files are generated."""
    test_files_dir = Path(__file__).parent / "test_files"
    
    if not test_files_dir.exists() or not any(test_files_dir.glob("*.tdms")):
        print("Generating benchmark test files...")
        from . import generate_test_files
        generate_test_files.main()


def print_summary(results: List[BenchmarkResult]):
    """Print a summary of benchmark results."""
    if not results:
        print("No benchmark results to summarize.")
        return
    
    print(f"\nBenchmark Summary ({len(results)} tests)")
    print("=" * 60)
    
    # Group by benchmark category
    by_category = {}
    for result in results:
        category = result.benchmark_name.split('_')[0]
        if category not in by_category:
            by_category[category] = []
        by_category[category].append(result)
    
    for category, category_results in by_category.items():
        print(f"\n{category.upper()} Benchmarks:")
        print("-" * 40)
        
        total_time = sum(r.time_sec for r in category_results)
        avg_throughput = sum(r.mb_per_sec for r in category_results if r.mb_per_sec > 0)
        avg_throughput = avg_throughput / len([r for r in category_results if r.mb_per_sec > 0]) if avg_throughput > 0 else 0
        
        print(f"  Tests: {len(category_results)}")
        print(f"  Total time: {total_time:.2f}s")
        if avg_throughput > 0:
            print(f"  Avg throughput: {avg_throughput:.1f} MB/s")
        
        # Show slowest operations
        slowest = sorted(category_results, key=lambda r: r.time_sec, reverse=True)[:3]
        print("  Slowest operations:")
        for result in slowest:
            print(f"    {result.operation}: {result.time_sec:.3f}s ({result.file_type})")


class FileTypeConfig:
    """Configuration for different file types used in benchmarks."""
    
    SMALL = {
        'name': 'small',
        'channels': [1, 5, 10],
        'samples': [1000, 10000],
        'target_size_mb': 1.0
    }
    
    MEDIUM = {
        'name': 'medium',
        'channels': [10, 50, 100],
        'samples': [100000, 1000000],
        'target_size_mb': 100.0
    }
    
    LARGE = {
        'name': 'large',
        'channels': [100, 500],
        'samples': [1000000, 10000000],
        'target_size_mb': 1000.0
    }
    
    MANY_CHANNELS = {
        'name': 'many_channels',
        'channels': [1000, 5000],
        'samples': [1000, 10000],
        'target_size_mb': 50.0
    }
    
    FEW_LARGE = {
        'name': 'few_large',
        'channels': [1, 3],
        'samples': [10000000, 50000000],
        'target_size_mb': 500.0
    }


def get_smoke_test_configs():
    """Get file configurations for smoke tests (CI-safe)."""
    return [FileTypeConfig.SMALL]


def get_full_test_configs():
    """Get all file configurations for comprehensive testing."""
    return [
        FileTypeConfig.SMALL,
        FileTypeConfig.MEDIUM,
        FileTypeConfig.LARGE,
        FileTypeConfig.MANY_CHANNELS,
        FileTypeConfig.FEW_LARGE
    ]


# Rust comparison notes for documentation
RUST_EQUIVALENTS = {
    'nptdms.TdmsFile.read()': 'TdmsFile::load()',
    'file["group"]["channel"]': 'file.get_channel("group", "channel")',
    'channel[:]': 'channel.as_f64() / channel.as_i32() / etc.',
    'channel.properties': 'channel.properties',
    'len(channel)': 'channel.data_len()',
    'channel.data': 'channel.data (enum variant)',
}