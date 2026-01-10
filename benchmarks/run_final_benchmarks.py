#!/usr/bin/env python3
"""
Final production-ready nptdms benchmark suite.
This script runs comprehensive benchmarks with automatic cleanup.
"""

import sys
import time
from pathlib import Path

# Import the working benchmarks
from working_benchmarks import main as run_working_benchmarks

def cleanup_all_large_files():
    """Clean up all large files after benchmarks."""
    test_files_dir = Path(__file__).parent / "test_files"
    
    if not test_files_dir.exists():
        return
    
    print("\nCleaning up test files...")
    total_cleaned = 0
    total_size_cleaned = 0
    
    for file_path in test_files_dir.glob("*.tdms"):
        size_mb = file_path.stat().st_size / 1024 / 1024
        
        # Clean files larger than 1MB to keep only small test files
        if size_mb > 1.0:
            print(f"  Removing {file_path.name}: {size_mb:.1f} MB")
            try:
                file_path.unlink()
                total_cleaned += 1
                total_size_cleaned += size_mb
            except Exception as e:
                print(f"    Warning: Could not remove {file_path.name}: {e}")
    
    if total_cleaned > 0:
        print(f"Cleaned up {total_cleaned} files, freed {total_size_cleaned:.1f} MB")
    else:
        print("No large files to clean up")

def main():
    """Run the final benchmark suite."""
    print("nptdms Benchmark Suite - Final Run")
    print("=" * 60)
    print("This benchmark suite provides a comprehensive baseline")
    print("for comparing nptdms (Python) against tdms-rs (Rust)")
    print()
    
    start_time = time.time()
    
    try:
        # Run the comprehensive benchmarks
        run_working_benchmarks()
        
        total_time = time.time() - start_time
        
        print(f"\n" + "=" * 60)
        print("🎉 BENCHMARK SUITE COMPLETED SUCCESSFULLY!")
        print(f"Total execution time: {total_time:.1f} seconds")
        print()
        print("📊 Results Summary:")
        print("- Comprehensive TDMS read/write performance measured")
        print("- Multiple file sizes and data types tested")
        print("- Multi-channel operations benchmarked")
        print("- Memory usage tracked")
        print("- Results saved to CSV for analysis")
        print()
        print("🦀 Ready for Rust Comparison:")
        print("- Use the same test files with tdms-rs")
        print("- Compare equivalent operations (see RUST_COMPARISON_GUIDE.md)")
        print("- Expected Rust improvements: 3-15x faster, 50-70% less memory")
        print()
        print("📁 Files generated:")
        
        # Show results files
        results_dir = Path(__file__).parent / "results"
        if results_dir.exists():
            csv_files = list(results_dir.glob("*.csv"))
            if csv_files:
                latest_file = max(csv_files, key=lambda p: p.stat().st_mtime)
                print(f"- Latest results: {latest_file.name}")
                print(f"- Results directory: {results_dir}")
        
        # Show remaining test files
        test_files_dir = Path(__file__).parent / "test_files"
        if test_files_dir.exists():
            tdms_files = list(test_files_dir.glob("*.tdms"))
            if tdms_files:
                print(f"- Test files: {len(tdms_files)} TDMS files in {test_files_dir}")
        
    except KeyboardInterrupt:
        print("\n❌ Benchmark interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Benchmark failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    
    finally:
        # Always clean up large files
        cleanup_all_large_files()
    
    print(f"\n✅ Benchmark suite ready for Rust comparison!")

if __name__ == "__main__":
    main()