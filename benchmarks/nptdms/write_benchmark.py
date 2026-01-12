import subprocess
import time
import os
import sys
import platform
import shutil
import numpy as np
from nptdms import TdmsWriter, ChannelObject, TdmsFile

DISKSPD_EXE = "diskspd.exe"
TEMP_FILE = "benchmark_write_test.tdms"
TEMP_DISK_FILE = "benchmark_disk_test.dat"

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
    print(f"Running raw disk write benchmark on {filename} using diskspd...")
    
    if not shutil.which("diskspd") and not os.path.exists(DISKSPD_EXE):
        print("ERROR: diskspd not found. Please download it and place it in the PATH or this directory.")
        return None, None

    # diskspd -w100 -b1M -c400M -Sh -d5 -o1 -t1 -L <testfile>
    # -w100: 100% Write
    # -b1M: Block size 1MB
    # -c400M: Create 400MB file
    # -Sh: No caching
    # -d5: Duration 5s
    cmd = ["diskspd", "-w100", "-b1M", "-c400M", "-Sh", "-d5", "-o1", "-t1", "-L", filename]
    cmd_str = " ".join(cmd)
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"diskspd failed: {result.stderr}")
            return None, cmd_str
            
        lines = result.stdout.splitlines()
        for line in lines:
            if line.strip().lower().startswith("total:"):
                parts = line.split("|")
                if len(parts) >= 3:
                    try:
                        mb_s_str = parts[2].strip()
                        mb_s = float(mb_s_str)
                        gb_s = mb_s / 1024.0
                        return gb_s, cmd_str
                    except ValueError:
                        continue
        return None, cmd_str
    except Exception as e:
        print(f"Error running diskspd: {e}")
        return None, cmd_str

def run_nptdms_benchmark():
    print("Running nptdms write benchmark...")
    
    # 50 million float64s ~= 400 MB (50e6 * 8 bytes = 400e6 bytes)
    # Using 52428800 to be exactly 400MB usually helpful or round number
    num_samples = 50_000_000 
    data = np.zeros(num_samples, dtype=np.float64)
    # Optional: Fill with random data or just keep zeros. Zeros are faster to generate.
    # User said "Data can be random floats or zeros"
    
    times = []
    
    for i in range(6):
        is_warmup = (i == 0)
        prefix = "WARMUP" if is_warmup else f"RUN {i}"
        
        # Remove file if it exists to measure creation + write
        if os.path.exists(TEMP_FILE):
            os.remove(TEMP_FILE)
            
        t0 = time.perf_counter()
        
        with TdmsWriter(TEMP_FILE) as tdms_writer:
            channel = ChannelObject("Group1", "Channel1", data)
            tdms_writer.write_segment([channel])
            
        t1 = time.perf_counter()
        dt = t1 - t0
        times.append(dt)
        print(f"{prefix}: {dt:.4f}s")
        
    measured_runs = times[1:]
    min_time = min(measured_runs)
    
    file_size_bytes = os.path.getsize(TEMP_FILE)
    file_size_gb = file_size_bytes / (1024**3)
    throughput_gb_s = file_size_gb / min_time
    
    # Clean up
    if os.path.exists(TEMP_FILE):
        os.remove(TEMP_FILE)
        
    return throughput_gb_s, min_time

def main():
    print("=== nptdms Full-Channel Write Benchmark ===")
    sys_info = get_system_info()
    
    raw_disk_gb_s = 0.0
    disk_cmd = "N/A"
    
    if platform.system() == "Windows":
        # Delete temp disk file if exists
        if os.path.exists(TEMP_DISK_FILE):
            try:
                os.remove(TEMP_DISK_FILE)
            except OSError:
                pass
        raw_disk_gb_s, disk_cmd = run_disk_benchmark_windows(TEMP_DISK_FILE)
        # Cleanup diskspd file
        if os.path.exists(TEMP_DISK_FILE):
            try:
                os.remove(TEMP_DISK_FILE)
            except OSError:
                pass
    else:
        print("Non-Windows OS detected. Skipping diskspd.")
        raw_disk_gb_s = float('nan')

    nptdms_gb_s, min_time = run_nptdms_benchmark()
    
    print("\n" + "="*40)
    print("WRITE BENCHMARK RESULTS")
    print("="*40)
    print(f"System: {sys_info['os']}")
    print(f"CPU:    {sys_info['cpu']}")
    print(f"Python: {sys_info['python']}")
    print(f"nptdms: {sys_info['nptdms']}")
    print("-" * 40)
    
    if raw_disk_gb_s is not None and not np.isnan(raw_disk_gb_s):
        print(f"RAW DISK WRITE (diskspd):   {raw_disk_gb_s:.2f} GB/s")
    else:
        print("RAW DISK WRITE (diskspd):   N/A")
        
    print(f"nptdms WRITE:               {nptdms_gb_s:.2f} GB/s (Min time: {min_time:.4f}s)")
    
    if raw_disk_gb_s is not None and not np.isnan(raw_disk_gb_s) and raw_disk_gb_s > 0:
        ratio = (nptdms_gb_s / raw_disk_gb_s) * 100
        print(f"RELATIVE PERF:              {ratio:.2f} %")
        
        if nptdms_gb_s > raw_disk_gb_s:
            print("\n[WARNING] nptdms speed > raw disk speed.")
            print("          This implies OS caching. Real valid disk I/O was not fully measured.")

    print("="*40)

if __name__ == "__main__":
    main()
