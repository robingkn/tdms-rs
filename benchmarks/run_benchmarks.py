"""
Main benchmark runner for nptdms performance testing.

This script orchestrates all benchmark categories and provides
both smoke testing (CI-safe) and full benchmark modes.
"""

import argparse
import sys
import time
from pathlib import Path
from typing import List

# Import all benchmark modules
try:
    from . import (
        generate_test_files,
        read_benchmarks,
        write_benchmarks,
        channel_access_benchmarks,
        stress_benchmarks
    )
    from .benchmark_utils import (
        get_benchmark_results, clear_benchmark_results,
        write_csv_results, write_json_results, print_summary
    )
except ImportError:
    # Handle direct execution
    import generate_test_files
    import read_benchmarks
    import write_benchmarks
    import channel_access_benchmarks
    import stress_benchmarks
    from benchmark_utils import (
        get_benchmark_results, clear_benchmark_results,
        write_csv_results, write_json_results, print_summary
    )


def run_smoke_tests():
    """
    Run smoke tests (CI-safe, ~5 minutes).
    
    Smoke tests include:
    - Small file operations only
    - Basic read/write operations
    - Essential channel access patterns
    - No stress tests or large files
    """
    print("Running Smoke Tests (CI Mode)")
    print("=" * 50)
    
    # Generate small test files only
    print("Generating small test files...")
    generate_test_files.main('smoke')
    
    # Clear any previous results
    clear_benchmark_results()
    
    # Run essential benchmarks with small files only
    print("\n1. Basic Read Operations...")
    read_benchmarks.benchmark_file_opening(Path(__file__).parent / "test_files")
    read_benchmarks.benchmark_single_channel_reads(Path(__file__).parent / "test_files")
    
    print("\n2. Basic Write Operations...")
    write_benchmarks.benchmark_single_channel_writes()
    
    print("\n3. Essential Channel Access...")
    channel_access_benchmarks.benchmark_channel_lookup_patterns(Path(__file__).parent / "test_files")
    channel_access_benchmarks.benchmark_data_access_patterns(Path(__file__).parent / "test_files")
    
    return get_benchmark_results()


def run_full_benchmarks():
    """
    Run comprehensive benchmark suite (~30-60 minutes).
    
    Full benchmarks include:
    - All file sizes and types
    - All benchmark categories
    - Stress tests and edge cases
    - Detailed profiling
    """
    print("Running Full Benchmark Suite")
    print("=" * 50)
    
    # Generate all test files
    print("Generating comprehensive test files...")
    generate_test_files.main('full')
    
    # Clear any previous results
    clear_benchmark_results()
    
    # Run all benchmark categories
    print("\n1. Read Benchmarks...")
    read_benchmarks.main()
    
    print("\n2. Write Benchmarks...")
    write_benchmarks.main()
    
    print("\n3. Channel Access Benchmarks...")
    channel_access_benchmarks.main()
    
    print("\n4. Stress Benchmarks...")
    stress_benchmarks.main()
    
    return get_benchmark_results()


def save_results(results: List, mode: str, output_format: str = 'csv'):
    """Save benchmark results to files."""
    if not results:
        print("No results to save.")
        return
    
    output_dir = Path(__file__).parent / "results"
    output_dir.mkdir(exist_ok=True)
    
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    
    if output_format in ['csv', 'both']:
        csv_file = output_dir / f"nptdms_benchmarks_{mode}_{timestamp}.csv"
        write_csv_results(results, csv_file)
        print(f"CSV results saved to: {csv_file}")
    
    if output_format in ['json', 'both']:
        json_file = output_dir / f"nptdms_benchmarks_{mode}_{timestamp}.json"
        write_json_results(results, json_file)
        print(f"JSON results saved to: {json_file}")


def main():
    """Main benchmark runner."""
    parser = argparse.ArgumentParser(
        description="nptdms Benchmark Suite - Baseline for tdms-rs comparison",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python run_benchmarks.py --mode smoke              # Quick CI-safe tests
  python run_benchmarks.py --mode full               # Comprehensive benchmarks
  python run_benchmarks.py --mode full --format json # Save as JSON
  python run_benchmarks.py --mode smoke --no-summary # Skip result summary
        """
    )
    
    parser.add_argument(
        '--mode',
        choices=['smoke', 'full'],
        default='smoke',
        help='Benchmark mode: smoke (fast, CI-safe) or full (comprehensive)'
    )
    
    parser.add_argument(
        '--format',
        choices=['csv', 'json', 'both'],
        default='csv',
        help='Output format for results'
    )
    
    parser.add_argument(
        '--no-summary',
        action='store_true',
        help='Skip printing result summary'
    )
    
    parser.add_argument(
        '--output-dir',
        type=Path,
        help='Custom output directory for results'
    )
    
    args = parser.parse_args()
    
    # Check dependencies
    try:
        import nptdms
        import numpy
        import psutil
    except ImportError as e:
        print(f"Error: Missing required dependency: {e}")
        print("Please install: pip install nptdms numpy psutil")
        sys.exit(1)
    
    # Record start time
    start_time = time.time()
    
    print("nptdms Benchmark Suite")
    print("=" * 60)
    print(f"Mode: {args.mode}")
    print(f"Output format: {args.format}")
    print(f"Start time: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    print()
    
    # Run benchmarks based on mode
    try:
        if args.mode == 'smoke':
            results = run_smoke_tests()
        else:
            results = run_full_benchmarks()
        
        # Calculate total time
        total_time = time.time() - start_time
        
        print(f"\nBenchmark Suite Complete!")
        print(f"Total time: {total_time:.1f} seconds")
        print(f"Total tests: {len(results)}")
        
        # Save results
        if args.output_dir:
            # Override output directory in benchmark_utils
            import benchmark_utils
            original_write_csv = benchmark_utils.write_csv_results
            original_write_json = benchmark_utils.write_json_results
            
            def custom_write_csv(results, filename):
                custom_path = args.output_dir / filename.name
                return original_write_csv(results, custom_path)
            
            def custom_write_json(results, filename):
                custom_path = args.output_dir / filename.name
                return original_write_json(results, custom_path)
            
            benchmark_utils.write_csv_results = custom_write_csv
            benchmark_utils.write_json_results = custom_write_json
        
        save_results(results, args.mode, args.format)
        
        # Print summary unless disabled
        if not args.no_summary:
            print_summary(results)
        
        # Success message
        print(f"\n✅ Benchmark suite completed successfully!")
        print(f"Results can be compared against Rust implementation (tdms-rs)")
        
    except KeyboardInterrupt:
        print("\n❌ Benchmark suite interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Benchmark suite failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()