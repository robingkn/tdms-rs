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

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", help="Output results in JSON")
    parser.add_argument("--file", type=str, default=DATA_FILE, help="Path to TDMS file")
    parser.add_argument("--samples", type=int, default=SAMPLE_COUNT, help="Number of samples")
    parser.add_argument("--iterations", type=int, default=5, help="Number of measured iterations")
    parser.add_argument("--warmup", type=int, default=1, help="Number of warmup iterations")
    args = parser.parse_args()
    
    # Update globals or pass them
    # Better to pass them to functions, but functions use globals currently.
    # Refactoring functions to accept args is cleaner.
    
    file_path = args.file
    sample_count = args.samples
    data_size_gb = (sample_count * 8) / 1e9
    
    if not args.json:
        print("=== nptdms Benchmark ===")
        print(f"File: {file_path}")
        print(f"Samples: {sample_count}")
        print(f"Size: {data_size_gb:.2f} GB")

    # Run Write
    write_gb_s, write_time = run_write_benchmark(file_path, sample_count, args.iterations, args.warmup, silent=args.json)

    # Run Read (uses file from Write)
    read_gb_s, read_time = run_read_benchmark(file_path, sample_count, args.iterations, args.warmup, silent=args.json)
    
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

def run_write_benchmark(filename, sample_count, iterations, warmup, silent=False):
    data_size_gb = (sample_count * 8) / 1e9
    if not silent:
        print(f"Running Write Benchmark on {filename}...")
    
    # Pre-generate data to exclude from timing
    if not silent:
        print("Generating data in memory...")
    data = np.arange(sample_count, dtype=np.float64)
    
    times = []
    
    for i in range(iterations + warmup):
        is_warmup = (i < warmup)
        prefix = "WARMUP" if is_warmup else f"RUN {i - warmup}"
        
        # Ensure fresh file
        if os.path.exists(filename):
            os.remove(filename)
            
        t0 = time.perf_counter()
        
        with TdmsWriter(filename) as tdms_writer:
            channel = ChannelObject("Group1", "Channel1", data)
            tdms_writer.write_segment([channel])
            
        dt = time.perf_counter() - t0
        
        if not is_warmup:
            times.append(dt)
            
        if not silent:
            print(f"{prefix}: {dt:.4f}s")
        
    min_time = min(times)
    throughput_gb_s = data_size_gb / min_time
    
    return throughput_gb_s, min_time

def run_read_benchmark(filename, sample_count, iterations, warmup, silent=False):
    data_size_gb = (sample_count * 8) / 1e9
    if not silent:
        print(f"Running Read Benchmark on {filename}...")
    
    if not os.path.exists(filename):
        if not silent:
            print("File not found, generating...")
        data = np.arange(sample_count, dtype=np.float64)
        with TdmsWriter(filename) as tdms_writer:
            channel = ChannelObject("Group1", "Channel1", data)
            tdms_writer.write_segment([channel])

    times = []
    
    for i in range(iterations + warmup):
        is_warmup = (i < warmup)
        prefix = "WARMUP" if is_warmup else f"RUN {i - warmup}"
        
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
    throughput_gb_s = data_size_gb / min_time
    
    return throughput_gb_s, min_time

if __name__ == "__main__":
    main()
