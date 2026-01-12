import subprocess
import time
import os
import sys
import platform
import re
import shutil
import numpy as np
from nptdms import TdmsFile

DATA_FILE = "benchmark_data.tdms"
DISKSPD_EXE = "diskspd.exe" # User should ensure this is in PATH or current dir

def get_system_info():
    info = {
        "os": platform.system() + " " + platform.release(),
        "cpu": platform.processor(),
        "python": sys.version.split()[0],
    }
    try:
        import nptdms
        info["nptdms"] = nptdms.__version__
    except ImportError:
        info["nptdms"] = "Not Installed"
    return info

def run_disk_benchmark_windows(filename):
    print(f"Running raw disk benchmark on {filename} using diskspd...")
    
    if not shutil.which("diskspd") and not os.path.exists(DISKSPD_EXE):
        print("ERROR: diskspd not found. Please download it and place it in the PATH or this directory.")
        print("Download: https://github.com/microsoft/diskspd")
        return None, None

    # diskspd -b1M -d10 -o1 -t1 -Sh -L <testfile>
    # -b1M: Block size 1MB, -d10: Duration 10s, -Sh: No caching
    cmd = ["diskspd", "-b1M", "-d10", "-o1", "-t1", "-Sh", "-L", filename]
    cmd_str = " ".join(cmd)
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"diskspd failed: {result.stderr}")
            return None, cmd_str
            
        lines = result.stdout.splitlines()
        for line in lines:
            # Check for total: line
            # Format: total: <bytes> | <ios> | <mib/s> | ...
            if line.strip().lower().startswith("total:"):
                parts = line.split("|")
                # parts[0] is "total: ... bytes ", parts[1] is IOs, parts[2] is MiB/s
                if len(parts) >= 3:
                    try:
                        mb_s_str = parts[2].strip()
                        mb_s = float(mb_s_str)
                        gb_s = mb_s / 1024.0
                        return gb_s, cmd_str
                    except ValueError:
                        continue
            
        print("Could not parse diskspd output. Full output:")
        print(result.stdout)
        return None, cmd_str
        
    except Exception as e:
        print(f"Error running diskspd: {e}")
        return None, cmd_str

def run_nptdms_benchmark(filename):
    print(f"Running nptdms benchmark on {filename}...")
    
    if not os.path.exists(filename):
        print(f"File {filename} not found. Run generate_file.py first.")
        return None
    
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
        times.append(dt)
        print(f"{prefix}: {dt:.4f}s")
        
    # Discard first (warmup), take best time
    measured_runs = times[1:]
    min_time = min(measured_runs)
    
    file_size_gb = os.path.getsize(filename) / (1024**3)
    throughput_gb_s = file_size_gb / min_time
    
    return throughput_gb_s, min_time

def main():
    print("=== Raw Disk vs nptdms Read Benchmark ===")
    
    if not os.path.exists(DATA_FILE):
        print("Data file not found. Please run generate_file.py.")
        return

    sys_info = get_system_info()
    
    raw_disk_gb_s = 0.0
    disk_cmd = "N/A"
    
    if platform.system() == "Windows":
        abs_path = os.path.abspath(DATA_FILE)
        raw_disk_gb_s, disk_cmd = run_disk_benchmark_windows(abs_path)
    else:
        print("Linux/Mac support not implemented.")
    
    if raw_disk_gb_s is None:
        print("Skipping Raw Disk comparison due to error.")
        raw_disk_gb_s = float('nan')

    nptdms_gb_s, min_time = run_nptdms_benchmark(DATA_FILE)
    
    if nptdms_gb_s is None:
        return

    print("\n" + "="*40)
    print("BENCHMARK RESULTS")
    print("="*40)
    print(f"System: {sys_info['os']}")
    print(f"CPU:    {sys_info['cpu']}")
    print(f"Python: {sys_info['python']}")
    print(f"nptdms: {sys_info['nptdms']}")
    print("-" * 40)
    
    if not np.isnan(raw_disk_gb_s):
        print(f"RAW DISK READ:     {raw_disk_gb_s:.2f} GB/s")
        print(f"Command: {disk_cmd.split(os.sep)[-1] if disk_cmd else 'N/A'}")
    else:
        print("RAW DISK READ:     FAILED/SKIPPED")

    print(f"nptdms READ:       {nptdms_gb_s:.2f} GB/s (Best of 5)")
    
    if not np.isnan(raw_disk_gb_s) and raw_disk_gb_s > 0:
        ratio = (nptdms_gb_s / raw_disk_gb_s) * 100
        print(f"RELATIVE PERF:     {ratio:.2f} %")
        
        if nptdms_gb_s > raw_disk_gb_s:
            print("\n[WARNING] nptdms speed > raw disk speed.")
            print("          This suggests nptdms was reading from OS RAM cache.")
            print("          Result measures Python+nptdms overhead, not disk I/O limit.")
            
    print("="*40)

if __name__ == "__main__":
    main()
