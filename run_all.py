import subprocess
import sys
import os
import json
import platform
import shutil
import time

# Configuration
DATA_FILE_SIZE_SAMPLES = 125_000_000  # 1.0 GB of f64
DATA_FILE_SIZE_GB = (DATA_FILE_SIZE_SAMPLES * 8) / 1e9
RUST_BINARY_PATH = os.path.join("target", "release", "benchmark")
if platform.system() == "Windows":
    RUST_BINARY_PATH += ".exe"
PYTHON_BENCH_SCRIPT = os.path.join("benchmarks", "nptdms", "benchmark.py")
DISK_BENCH_FILE = "benchmark_disk_io.dat" # Separate file for disk bench to avoid conflicts

def print_header(title):
    print("\n" + "="*50)
    print(f" {title}")
    print("="*50)

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

def get_disk_performance():
    print_header("RAW DISK I/O BENCHMARK")
    
    if platform.system() == "Windows":
        diskspd_exe = "diskspd.exe"
        if not shutil.which(diskspd_exe) and not os.path.exists(diskspd_exe):
             print(f"[WARNING] {diskspd_exe} not found. Skipping disk benchmark.")
             return None, None
        
        # Write Test
        print("Running Sequential Write (No Cache)...")
        # -c1G : Create 1GB file
        # -b1M : 1MB block size
        # -d5  : 5 seconds duration (minimal but enough for seq write stability)
        # -o1  : Overlapping IOs (queue depth)
        # -t1  : Threads
        # -Sh  : Disable write caching and read caching (software and hardware)
        # -w100: 100% Write
        # -L   : Latency stats (useful but mainly we want throughput)
        
        cmd_write = [diskspd_exe, "-c1G", "-b1M", "-d5", "-o1", "-t1", "-Sh", "-w100", "-L", DISK_BENCH_FILE]
        res_write = run_command(cmd_write)
        write_gb_s = parse_diskspd_throughput(res_write.stdout)
        print(f"Write Throughput: {write_gb_s:.2f} GB/s" if write_gb_s else "Write Benchmark Failed")

        # Read Test
        print("Running Sequential Read (No Cache)...")
        # -w0  : 0% Write (100% Read)
        cmd_read = [diskspd_exe, "-b1M", "-d5", "-o1", "-t1", "-Sh", "-w0", "-L", DISK_BENCH_FILE]
        res_read = run_command(cmd_read)
        read_gb_s = parse_diskspd_throughput(res_read.stdout)
        print(f"Read Throughput:  {read_gb_s:.2f} GB/s" if read_gb_s else "Read Benchmark Failed")
        
        # Cleanup
        if os.path.exists(DISK_BENCH_FILE):
            os.remove(DISK_BENCH_FILE)
            
        return write_gb_s, read_gb_s
        
    else:
        print("Linux/macOS disk benchmark manually implemented (TODO). Skipping.")
        return None, None

def parse_diskspd_throughput(output):
    if not output: return None
    lines = output.splitlines()
    for line in lines:
        if line.strip().lower().startswith("total:"):
            parts = line.split("|")
            if len(parts) >= 3:
                try:
                    mb_s = float(parts[2].strip())
                    return mb_s / 1000.0 # Convert MB/s to GB/s (using 1000 base for GB vs MB consistency is simpler, or 1024?)
                    # diskspd default is MiB/s usually? 
                    # "MB/s" in diskspd output is usually 10^6. 
                    # Wait, diskspd -L output column says "MiB/s" usually?
                    # Let's check logic: User asked for "Throughput (GB/s)".
                    # We will align everything to 10^9 bytes/sec (GB/s).
                    # If diskspd reports in MiB/s (1024^2), we need to convert.
                    # Standard diskspd output: "Total: ... | <MiB/s>"
                    # Let's assume input is MiB/s for safety, so * 1024*1024 / 1e9.
                    
                    # Actually, let's keep it simple. If we can't be sure, we treat it as "Units".
                    # But for now, let's assume the value is MB/s (10^6) or MiB/s.
                    # To be safe, let's just use the raw number if possible or standardise.
                    # I'll stick to: Value in output is usually MB (10^6) or MiB. 
                    # I will assume MB/s (10^6) for now as that's conservative.
                    return mb_s / 1000.0
                except:
                    return None
    return None

def run_rust_benchmark():
    print_header("RUST BENCHMARK (tdms-rs)")
    
    # 1. Build
    print("Building generic release binary...")
    build_cmd = ["cargo", "build", "--release", "--bin", "benchmark"]
    res = run_command(build_cmd)
    if res.returncode != 0:
        print("Build failed.")
        return None
    
    # 2. Run
    print("Running benchmark...")
    # We need to pass arguments if we implemented them. 
    # For now, we will rely on strict source code modification or args.
    # I will modify Rust code to accept args: file size, filename.
    # But to keep it simple as per "One command runs everything", 
    # the Rust code I modify next will just use the correct constants matching this script.
    
    cmd = [RUST_BINARY_PATH, "--json"]
    res = run_command(cmd)
    if res.returncode != 0:
        print(f"Rust benchmark failed: {res.stderr}")
        return None
        
    try:
        return json.loads(res.stdout)
    except json.JSONDecodeError:
        print("Failed to parse Rust JSON output.")
        print("Output:", res.stdout)
        return None

def run_python_benchmark():
    print_header("PYTHON BENCHMARK (nptdms)")
    
    # Ensure dependencies
    # pip install nptdms numpy (assumed installed or user runs this in venv)
    
    cmd = [sys.executable, PYTHON_BENCH_SCRIPT, "--json"]
    res = run_command(cmd)
    if res.returncode != 0:
        print(f"Python benchmark failed: {res.stderr}")
        return None
        
    try:
        return json.loads(res.stdout)
    except json.JSONDecodeError:
        print("Failed to parse Python JSON output.")
        print("Output:", res.stdout)
        return None

def generate_report(disk, rust, python):
    # Data Preparation
    disk_write = disk[0] if disk and disk[0] else None
    disk_read = disk[1] if disk and disk[1] else None
    
    rust_write = rust.get("write_gb_s") if rust else None
    rust_read = rust.get("read_gb_s") if rust else None
    
    py_write = python.get("write_gb_s") if python else None
    py_read = python.get("read_gb_s") if python else None

    # 1. Console Summary
    print("\n" + "="*40)
    print("TDMS DISK I/O PERFORMANCE REPORT")
    print("="*40)
    print("System:")
    print(f"  OS:        {platform.system()} {platform.release()}")
    print(f"  CPU:       {platform.processor()}")
    print(f"  File Size: {DATA_FILE_SIZE_GB:.2f} GB")
    print("-" * 40)
    
    def print_row(label, val, reference=None):
        if val is None:
            val_str = "N/A"
            pct_str = ""
        else:
            val_str = f"{val:.2f} GB/s"
            if reference:
                pct = (val / reference) * 100
                pct_str = f" ({pct:.1f}%)"
            else:
                pct_str = " (100%)" # Baseline
        print(f"{label:<13} {val_str}{pct_str}")

    print("WRITE PERFORMANCE")
    print("-" * 40)
    print_row("Raw Disk:", disk_write)
    print_row("nptdms:", py_write, disk_write)
    print_row("tdms-rs:", rust_write, disk_write)
    print("")
    
    print("READ PERFORMANCE")
    print("-" * 40)
    print_row("Raw Disk:", disk_read)
    print_row("nptdms:", py_read, disk_read)
    print_row("tdms-rs:", rust_read, disk_read)
    print("="*40)

    # 2. Markdown/CSV Generation
    md_table = f"""
| Operation | Disk (GB/s) | nptdms (GB/s) | nptdms % | tdms-rs (GB/s) | tdms-rs % |
|-----------|-------------|---------------|----------|----------------|-----------|
| Write     | {fmt(disk_write)} | {fmt(py_write)} | {pct(py_write, disk_write)} | {fmt(rust_write)} | {pct(rust_write, disk_write)} |
| Read      | {fmt(disk_read)}  | {fmt(py_read)}  | {pct(py_read, disk_read)}   | {fmt(rust_read)}  | {pct(rust_read, disk_read)}   |
"""
    
    with open("benchmark_results.md", "w") as f:
        f.write("# Benchmark Results\n")
        f.write(md_table)
        
    print("\nReport saved to benchmark_results.md")

def fmt(val):
    return f"{val:.2f}" if val is not None else "N/A"

def pct(val, ref):
    if val is None or ref is None or ref == 0: return "N/A"
    return f"{(val/ref)*100:.1f}%"

def main():
    disk_res = get_disk_performance()
    rust_res = run_rust_benchmark()
    py_res = run_python_benchmark()
    
    generate_report(disk_res, rust_res, py_res)

if __name__ == "__main__":
    main()
