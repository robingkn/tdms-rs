import sys
import os
import subprocess
import shutil
import platform

# Ensure utils is importable
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from utils import clobber_cache

def run_command(cmd, cwd=None, capture_output=True):
    try:
        result = subprocess.run(
            cmd, 
            cwd=cwd, 
            capture_output=capture_output, 
            text=True, 
            check=False 
        )
        return result
    except Exception as e:
        print(f"Error running command {cmd}: {e}")
        return None

def parse_diskspd_throughput(output):
    """
    Parses diskspd output to extract throughput in GB/s.
    diskspd reports MiB/s ($1024^2$), we normalize to GB/s ($10^9$).
    """
    if not output: return None
    lines = output.splitlines()
    for line in lines:
        if line.strip().lower().startswith("total:"):
            parts = line.split("|")
            if len(parts) >= 3:
                try:
                    mib_s = float(parts[2].strip())
                    # Convert MiB/s to B/s then to GB/s (decimal)
                    gb_s = (mib_s * 1024 * 1024) / 1e9
                    return gb_s
                except:
                    return None
    return None

def run_disk_benchmark(config):
    """
    Runs diskspd to measure raw disk write and read performance.
    Returns a tuple (write_gb_s, read_gb_s).
    """
    bench_file = config['paths']['disk_bench_file']
    file_size_gb = config['file_size_gb']
    
    # Ensure directory exists
    os.makedirs(os.path.dirname(bench_file), exist_ok=True)

    if platform.system() != "Windows":
        print("[WARNING] diskspd is only supported on Windows. Skipping disk benchmark.")
        return None, None

    diskspd_exe = "diskspd.exe"
    if not shutil.which(diskspd_exe) and not os.path.exists(diskspd_exe):
             print(f"[WARNING] {diskspd_exe} not found. Skipping disk benchmark.")
             return None, None

    # Write Test
    print("Running Raw Disk Write Benchmark (Cold Cache via Clobber)...")
    clobber_cache(file_size_gb)
    
    # -c1G: Create file
    # -b1M: 1MB block size
    # -d5: 5 seconds duration
    # -o1: Overlapping IOs
    # -t1: Threads
    # -w100: 100% Write
    # NO -Sh (Allow OS caching)
    
    size_param = f"-c{int(file_size_gb * 1000)}M" # Use decimal M for file creation if possible, but 1024M is fine for diskspd
    cmd_write = [diskspd_exe, size_param, "-b1M", "-d5", "-o1", "-t1", "-w100", "-L", bench_file]
    res_write = run_command(cmd_write)
    write_gb_s = parse_diskspd_throughput(res_write.stdout)
    
    # Read Test
    print("Running Raw Disk Read Benchmark (Cold Cache via Clobber)...")
    clobber_cache(file_size_gb)
    
    # -w0: 0% Write (100% Read)
    cmd_read = [diskspd_exe, "-b1M", "-d5", "-o1", "-t1", "-w0", "-L", bench_file]
    res_read = run_command(cmd_read)
    read_gb_s = parse_diskspd_throughput(res_read.stdout)
    
    if os.path.exists(bench_file):
        os.remove(bench_file)
        
    return write_gb_s, read_gb_s

if __name__ == "__main__":
    # Make it runnable standalone for testing
    import yaml
    # distinct config for standalone run
    mock_config = {
        'file_size_gb': 1.0,
        'paths': {'disk_bench_file': 'benchmark/data/disk_bench_standalone.dat'}
    }
    w, r = run_disk_benchmark(mock_config)
    print(f"Write: {w} GB/s, Read: {r} GB/s")
