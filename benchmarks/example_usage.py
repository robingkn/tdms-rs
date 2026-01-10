#!/usr/bin/env python3
"""
Example usage of the nptdms benchmark suite.

This script demonstrates how to run individual benchmarks
and work with the results programmatically.
"""

import sys
from pathlib import Path

# Add benchmarks directory to path
sys.path.insert(0, str(Path(__file__).parent))

from benchmark_utils import (
    benchmark_context, get_benchmark_results, clear_benchmark_results,
    write_csv_results, print_summary
)
import generate_test_files


def example_custom_benchmark():
    """Example of creating a custom benchmark."""
    print("Running custom benchmark example...")
    
    # Ensure we have test files
    test_files_dir = Path(__file__).parent / "test_files"
    if not test_files_dir.exists() or not any(test_files_dir.glob("*.tdms")):
        print("Generating test files...")
        generate_test_files.main('smoke')
    
    # Clear previous results
    clear_benchmark_results()
    
    # Example benchmark using context manager
    try:
        import nptdms
        
        for tdms_file in test_files_dir.glob("small_*.tdms"):
            file_size_mb = tdms_file.stat().st_size / 1024 / 1024
            
            with benchmark_context(
                "example_benchmark",
                "small",
                1,  # channels (will be updated)
                0,  # samples (will be updated)
                "mixed",
                "custom_operation",
                file_size_mb,
                "Example custom benchmark operation"
            ):
                # Your benchmark code here
                tdms_file_obj = nptdms.TdmsFile.read(str(tdms_file))
                
                # Count actual channels and samples
                total_channels = 0
                total_samples = 0
                
                for group in tdms_file_obj.groups():
                    for channel in group.channels():
                        total_channels += 1
                        if hasattr(channel, 'data') and channel.data is not None:
                            total_samples += len(channel.data)
                            # Access the data to measure actual read time
                            _ = channel[:]
                
                # Update the result with actual counts
                if hasattr(benchmark_context, '_results') and benchmark_context._results:
                    last_result = benchmark_context._results[-1]
                    last_result.channels = total_channels
                    last_result.samples = total_samples
    
    except ImportError:
        print("nptdms not installed. Install with: pip install nptdms")
        return
    
    # Get and display results
    results = get_benchmark_results()
    print(f"\nCompleted {len(results)} custom benchmarks")
    
    # Print summary
    print_summary(results)
    
    # Save results
    output_dir = Path(__file__).parent / "results"
    write_csv_results(results, output_dir / "example_results.csv")
    print(f"Results saved to: {output_dir / 'example_results.csv'}")


def example_analyze_results():
    """Example of analyzing benchmark results."""
    print("Analyzing benchmark results...")
    
    results_dir = Path(__file__).parent / "results"
    csv_files = list(results_dir.glob("*.csv"))
    
    if not csv_files:
        print("No result files found. Run some benchmarks first.")
        return
    
    import csv
    
    # Read the most recent results file
    latest_file = max(csv_files, key=lambda p: p.stat().st_mtime)
    print(f"Analyzing: {latest_file}")
    
    with open(latest_file, 'r') as f:
        reader = csv.DictReader(f)
        results = list(reader)
    
    print(f"Total benchmarks: {len(results)}")
    
    # Analyze by category
    categories = {}
    for result in results:
        category = result['benchmark_name']
        if category not in categories:
            categories[category] = []
        categories[category].append(float(result['time_sec']))
    
    print("\nPerformance by category:")
    for category, times in categories.items():
        avg_time = sum(times) / len(times)
        max_time = max(times)
        min_time = min(times)
        print(f"  {category}:")
        print(f"    Average: {avg_time:.3f}s")
        print(f"    Range: {min_time:.3f}s - {max_time:.3f}s")
        print(f"    Tests: {len(times)}")
    
    # Find slowest operations
    print("\nSlowest operations:")
    sorted_results = sorted(results, key=lambda r: float(r['time_sec']), reverse=True)
    for result in sorted_results[:5]:
        print(f"  {result['operation']}: {result['time_sec']}s ({result['file_type']})")
    
    # Find highest throughput operations
    throughput_results = [r for r in results if float(r['mb_per_sec']) > 0]
    if throughput_results:
        print("\nHighest throughput operations:")
        sorted_throughput = sorted(throughput_results, key=lambda r: float(r['mb_per_sec']), reverse=True)
        for result in sorted_throughput[:5]:
            print(f"  {result['operation']}: {result['mb_per_sec']} MB/s ({result['file_type']})")


def example_compare_results():
    """Example of comparing two benchmark runs."""
    print("Comparing benchmark results...")
    
    results_dir = Path(__file__).parent / "results"
    csv_files = sorted(results_dir.glob("*.csv"), key=lambda p: p.stat().st_mtime)
    
    if len(csv_files) < 2:
        print("Need at least 2 result files for comparison.")
        return
    
    import csv
    
    # Compare the two most recent files
    older_file = csv_files[-2]
    newer_file = csv_files[-1]
    
    print(f"Comparing:")
    print(f"  Baseline: {older_file.name}")
    print(f"  Current:  {newer_file.name}")
    
    # Read both files
    def read_results(file_path):
        with open(file_path, 'r') as f:
            reader = csv.DictReader(f)
            return {r['operation']: float(r['time_sec']) for r in reader}
    
    baseline = read_results(older_file)
    current = read_results(newer_file)
    
    # Compare common operations
    common_ops = set(baseline.keys()) & set(current.keys())
    
    if not common_ops:
        print("No common operations found between files.")
        return
    
    print(f"\nComparing {len(common_ops)} common operations:")
    print("Operation | Baseline (s) | Current (s) | Change")
    print("-" * 50)
    
    improvements = []
    regressions = []
    
    for op in sorted(common_ops):
        baseline_time = baseline[op]
        current_time = current[op]
        change_pct = ((current_time - baseline_time) / baseline_time) * 100
        
        status = "📈" if change_pct > 5 else "📉" if change_pct < -5 else "➡️"
        print(f"{op[:20]:20} | {baseline_time:8.3f} | {current_time:8.3f} | {status} {change_pct:+5.1f}%")
        
        if change_pct > 10:
            regressions.append((op, change_pct))
        elif change_pct < -10:
            improvements.append((op, change_pct))
    
    if improvements:
        print(f"\n🎉 Improvements ({len(improvements)}):")
        for op, change in improvements:
            print(f"  {op}: {change:+.1f}%")
    
    if regressions:
        print(f"\n⚠️ Regressions ({len(regressions)}):")
        for op, change in regressions:
            print(f"  {op}: {change:+.1f}%")


def main():
    """Main example function."""
    print("nptdms Benchmark Suite - Example Usage")
    print("=" * 50)
    
    # Check if nptdms is available
    try:
        import nptdms
        print(f"✅ nptdms version: {nptdms.__version__}")
    except ImportError:
        print("❌ nptdms not installed. Install with: pip install nptdms")
        return
    
    print("\n1. Running custom benchmark example...")
    example_custom_benchmark()
    
    print("\n2. Analyzing results...")
    example_analyze_results()
    
    print("\n3. Comparing results (if multiple files exist)...")
    example_compare_results()
    
    print("\n" + "=" * 50)
    print("Example complete! Try these commands:")
    print("  python run_benchmarks.py --mode smoke    # Quick tests")
    print("  python run_benchmarks.py --mode full     # Full suite")
    print("  python setup_benchmarks.py               # Setup validation")


if __name__ == "__main__":
    main()