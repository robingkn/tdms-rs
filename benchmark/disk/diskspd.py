import subprocess
import os
import shutil
import platform

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
    Assumes diskspd output shows MB/s or MiB/s, we normalize to GB/s (10^9).
    """
    if not output: return None
    lines = output.splitlines()
    for line in lines:
        if line.strip().lower().startswith("total:"):
            parts = line.split("|")
            if len(parts) >= 3:
                try:
                    # Diskspd usually reports in MB/s (10^6) or MiB/s.
                    # We will treat the number as MB/s and convert to GB/s.
                    # This is an approximation based on typical diskspd output.
                    mb_s = float(parts[2].strip())
                    return mb_s / 1000.0 
                except:
                    return None
    return None

def run_disk_benchmark(config):
    """
    Runs diskspd to measure raw disk write and read performance.
    Returns a tuple (write_gb_s, read_gb_s).
    """
    bench_file = config['paths']['disk_bench_file']
    # Ensure directory exists
    os.makedirs(os.path.dirname(bench_file), exist_ok=True)

    if platform.system() != "Windows":
        print("[WARNING] diskspd is only supported on Windows. Skipping disk benchmark.")
        return None, None

    diskspd_exe = "diskspd.exe"
    # Check if diskspd is in PATH or in strict locations if needed
    if not shutil.which(diskspd_exe) and not os.path.exists(diskspd_exe):
         # Try looking in tools/ if it exists there, but user didn't specify tools/ location for diskspd.
         # We will assume it's in PATH or current root.
         pass
    
    if not shutil.which(diskspd_exe) and not os.path.exists(diskspd_exe):
             print(f"[WARNING] {diskspd_exe} not found. Skipping disk benchmark.")
             return None, None

    # Write Test
    print("Running Raw Disk Write Benchmark (No Cache)...")
    # -c1G : Create 1GB file (using config logic if possible, but keep simple flags for robust CLI)
    # We will use the file size from config if we can pass it to diskspd properly. 
    # diskspd -c takes string like 1G, 100M.
    size_param = f"-c{int(config['file_size_gb'] * 1024)}M" # Convert GB to MB for flag
    
    # -b1M : 1MB block size
    # -d5  : 5 seconds duration
    # -o1  : Overlapping IOs
    # -t1  : Threads
    # -Sh  : Disable write/read caching
    # -w100: 100% Write
    # -L   : Latency stats
    
    cmd_write = [diskspd_exe, size_param, "-b1M", "-d5", "-o1", "-t1", "-Sh", "-w100", "-L", bench_file]
    res_write = run_command(cmd_write)
    write_gb_s = parse_diskspd_throughput(res_write.stdout)
    
    # Read Test
    print("Running Raw Disk Read Benchmark (No Cache)...")
    # -w0  : 0% Write (100% Read)
    # No -c flag because file exists (though -c creates/recreates)
    # diskspd creates file if not exists or if -c is provided.
    # We can just reuse the file.
    cmd_read = [diskspd_exe, "-b1M", "-d5", "-o1", "-t1", "-Sh", "-w0", "-L", bench_file]
    res_read = run_command(cmd_read)
    read_gb_s = parse_diskspd_throughput(res_read.stdout)
    
    # Cleanup only if needed, but per specs we clean up data
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
