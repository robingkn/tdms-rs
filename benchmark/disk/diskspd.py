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
    iterations = config.get('iterations', 5)
    warmup = config.get('warmup_iterations', 1)
    
    # Ensure directory exists
    os.makedirs(os.path.dirname(bench_file), exist_ok=True)

    if platform.system() != "Windows":
        print("[WARNING] diskspd is only supported on Windows. Skipping disk benchmark.")
        return None, None

    diskspd_exe = "diskspd.exe"
    if not shutil.which(diskspd_exe) and not os.path.exists(diskspd_exe):
             print(f"[WARNING] {diskspd_exe} not found. Skipping disk benchmark.")
             return None, None

    # Calculate number of 1MB blocks for fixed-size IO
    # 1 GB = 1,000,000,000 bytes. 
    # nptdms/tdms-rs use 125M samples * 8 = 1,000,000,000 bytes.
    # diskspd -b1M uses 1024*1024 = 1,048,576 bytes.
    # To get as close to 10^9 as possible: 10^9 / 1048576 = 953.67 blocks.
    # We will use 1000 blocks of 1,000,000 bytes if diskspd supports it, 
    # but diskspd -b usually expects power of 2.
    # Let's use -b1000000 if possible, or just accept the tiny MiB/MB diff 
    # and normalize carefully.
    
    # Use -b1M (1,048,576 bytes) and -n 954 blocks ~= 1,000,344,064 bytes
    n_blocks = int((file_size_gb * 1e9) / (1024 * 1024))
    
    def run_trial(mode):
        times = []
        for i in range(iterations + warmup):
            is_warmup = (i < warmup)
            prefix = "WARMUP" if is_warmup else f"RUN {i - warmup}"
            
            clobber_cache(file_size_gb)
            
            # -c: Create file (only on first write)
            # -b1M: 1MB block size
            # -n: number of blocks (fixed task)
            # -o1: Overlapping IOs
            # -t1: Threads
            # -Sh: Software cache disabled
            # -D: Hardware cache/write-through
            # -L: Latency/Progress
            
            if mode == 'write':
                # Re-create file every time to be fair with library cold-create
                if os.path.exists(bench_file): os.remove(bench_file)
                cmd = [diskspd_exe, f"-c{int(file_size_gb * 1000)}M", "-b1M", f"-n{n_blocks}", "-o1", "-t1", "-w100", "-Sh", "-D", "-L", bench_file]
            else:
                cmd = [diskspd_exe, "-b1M", f"-n{n_blocks}", "-o1", "-t1", "-w0", "-Sh", "-D", "-L", bench_file]
            
            res = run_command(cmd)
            gb_s = parse_diskspd_throughput(res.stdout)
            
            if not is_warmup and gb_s:
                times.append(gb_s)
            
            if gb_s:
                print(f"{mode.capitalize()} {prefix}: {gb_s:.2f} GB/s")
            else:
                print(f"{mode.capitalize()} {prefix}: FAILED")
                
        return max(times) if times else None

    print(f"Running Raw Disk Write Benchmark ({iterations} runs, Unbuffered)...")
    write_gb_s = run_trial('write')
    
    print(f"\nRunning Raw Disk Read Benchmark ({iterations} runs, Unbuffered)...")
    read_gb_s = run_trial('read')
    
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
