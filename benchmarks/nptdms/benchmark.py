import time
import os
import sys
import argparse
import json
import numpy as np
from nptdms import TdmsFile, TdmsWriter, ChannelObject, GroupObject

# Configuration
DATA_FILE = "benchmark_python.tdms"
SAMPLE_COUNT = 125_000_000  # 1.0 GB of f64
DATA_SIZE_GB = (SAMPLE_COUNT * 8) / 1e9

def run_write_benchmark(filename, silent=False):
    if not silent:
        print(f"Running Write Benchmark on {filename}...")
    
    # Pre-generate data to exclude from timing
    if not silent:
        print("Generating data in memory...")
    data = np.arange(SAMPLE_COUNT, dtype=np.float64)
    
    times = []
    
    # 5 runs + 1 warmup
    for i in range(6):
        is_warmup = (i == 0)
        prefix = "WARMUP" if is_warmup else f"RUN {i}"
        
        # Ensure fresh file
        if os.path.exists(filename):
            os.remove(filename)
            
        t0 = time.perf_counter()
        
        with TdmsWriter(filename) as tdms_writer:
            channel = ChannelObject("Group1", "Channel1", data)
            tdms_writer.write_segment([channel])
            # TdmsWriter context manager closes file which flushes buffers.
            # To be strict about "fsync", standard close usually flushes to OS cache.
            # Python's `open` defaults to buffered, but TdmsWriter handles it.
            # Unlike Rust's explicit flush, we rely on close().
            
        dt = time.perf_counter() - t0
        
        if not is_warmup:
            times.append(dt)
            
        if not silent:
            print(f"{prefix}: {dt:.4f}s")
        
    min_time = min(times)
    throughput_gb_s = DATA_SIZE_GB / min_time
    
    return throughput_gb_s, min_time

def run_read_benchmark(filename, silent=False):
    if not silent:
        print(f"Running Read Benchmark on {filename}...")
    
    if not os.path.exists(filename):
        # Should populate from write bench, but if not:
        if not silent:
            print("File not found, generating...")
        data = np.arange(SAMPLE_COUNT, dtype=np.float64)
        with TdmsWriter(filename) as tdms_writer:
            channel = ChannelObject("Group1", "Channel1", data)
            tdms_writer.write_segment([channel])

    times = []
    
    # 5 runs + 1 warmup
    for i in range(6):
        is_warmup = (i == 0)
        prefix = "WARMUP" if is_warmup else f"RUN {i}"
        
        t0 = time.perf_counter()
        
        with TdmsFile.read(filename) as tdms_file:
            group = tdms_file["Group1"]
            channel = group["Channel1"]
            # Full read
            data = channel[:]
            
            # Touch data to prevent DCE
            _ = data[0] + data[-1]
            
        dt = time.perf_counter() - t0
        
        if not is_warmup:
            times.append(dt)
            
        if not silent:
            print(f"{prefix}: {dt:.4f}s")
        
    min_time = min(times)
    throughput_gb_s = DATA_SIZE_GB / min_time
    
    return throughput_gb_s, min_time

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", help="Output results in JSON")
    args = parser.parse_args()
    
    if not args.json:
        print("=== nptdms Benchmark ===")
        print(f"Samples: {SAMPLE_COUNT}")
        print(f"Size: {DATA_SIZE_GB:.2f} GB")

    # Run Write
    write_gb_s, write_time = run_write_benchmark(DATA_FILE, silent=args.json)

    # Run Read (uses file from Write)
    read_gb_s, read_time = run_read_benchmark(DATA_FILE, silent=args.json)
    
    # Cleanup
    if os.path.exists(DATA_FILE):
        os.remove(DATA_FILE)

    if args.json:
        result = {
            "write_gb_s": write_gb_s,
            "write_min_time": write_time,
            "read_gb_s": read_gb_s,
            "read_min_time": read_time
        }
        print(json.dumps(result))
    else:
        print("\n=== SUMMARY ===")
        print(f"nptdms Write: {write_gb_s:.2f} GB/s ({write_time:.4f}s)")
        print(f"nptdms Read:  {read_gb_s:.2f} GB/s ({read_time:.4f}s)")

if __name__ == "__main__":
    main()
