import sys
import os
import subprocess
import json
import platform
import shutil
import time

# Try to import diskspd module
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from disk import diskspd

def load_config(config_path="benchmark/config.yaml"):
    """
    Simple YAML parser for the specific structure of config.yaml.
    Falls back to basic line parsing to avoid PyYAML dependency if not installed.
    """
    try:
        import yaml
        with open(config_path, "r") as f:
            return yaml.safe_load(f)
    except ImportError:
        print("[INFO] PyYAML not found, using simple fallback parser.")
        config = {}
        current_section = None
        with open(config_path, "r") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if line.endswith(":"):
                    current_section = line[:-1]
                    config[current_section] = {}
                elif ":" in line:
                    key, val = line.split(":", 1)
                    key = key.strip()
                    val = val.strip().strip('"').strip("'")
                    
                    # Type conversion
                    if "." in val and val.replace(".", "").isdigit():
                        val = float(val)
                    elif val.isdigit():
                        val = int(val)
                    
                    if current_section:
                        config[current_section][key] = val
                    else:
                        config[key] = val
        return config

def print_header(title):
    print("\n" + "="*50)
    print(f" {title}")
    print("="*50)

def run_command(cmd, cwd=None, capture_output=True):
    try:
        # result = subprocess.run(cmd, cwd=cwd, capture_output=capture_output, text=True, check=False)
        # For live output if not capturing
        if not capture_output:
            return subprocess.run(cmd, cwd=cwd, check=False)
        return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    except Exception as e:
        print(f"Error running command {cmd}: {e}")
        return None

def main():
    print_header("TDMS BENCHMARK SUITE")
    
    # Load Config
    config_path = os.path.join(os.path.dirname(__file__), "config.yaml")
    if not os.path.exists(config_path):
        print(f"[ERROR] Config file not found at {config_path}")
        sys.exit(1)
        
    config = load_config(config_path)
    print(f"Configuration loaded: {config['file_size_gb']} GB, {config['samples']} samples\n")

    # Ensure directories
    os.makedirs(config['paths']['results_dir'], exist_ok=True)
    os.makedirs("benchmark/data", exist_ok=True)

    # 1. Disk Benchmark
    print_header("1. RAW DISK BENCHMARK")
    disk_write, disk_read = diskspd.run_disk_benchmark(config)
    
    if disk_write:
        print(f"Disk Write: {disk_write:.2f} GB/s")
        print(f"Disk Read:  {disk_read:.2f} GB/s")
    else:
        print("Disk benchmark skipped or failed.")

    # 2. Rust Benchmark
    print_header("2. RUST BENCHMARK (tdms-rs)")
    
    # Build
    print("Building generic release binary...")
    build_cmd = ["cargo", "build", "--release", "--bin", "benchmark"]
    res = run_command(build_cmd)
    if res.returncode != 0:
        print(f"Build failed: {res.stderr}")
        rust_res = None
    else:
        print("Running benchmark...")
        # Path to binary
        binary_name = "benchmark.exe" if platform.system() == "Windows" else "benchmark"
        binary_path = os.path.join("target", "release", binary_name)
        
        rust_args = [
            binary_path, 
            "--json",
            "--file", config['paths']['tdms_file'],
            "--samples", str(config['samples']),
            "--iterations", str(config['iterations']),
            "--warmup", str(config.get('warmup_iterations', 1))
        ]
        
        res = run_command(rust_args)
        if res.returncode != 0:
            print(f"Rust benchmark failed: {res.stderr}")
            print(f"Output: {res.stdout}")
            rust_res = None
        else:
            try:
                rust_res = json.loads(res.stdout)
                print(f"Write: {rust_res['write_gb_s']:.2f} GB/s")
                print(f"Read:  {rust_res['read_gb_s']:.2f} GB/s")
            except Exception as e:
                print(f"Failed to parse Rust output: {e}")
                print(res.stdout)
                rust_res = None

    # 3. Python Benchmark
    print_header("3. PYTHON BENCHMARK (nptdms)")
    
    # Ensure nptdms installed? User responsibility provided environment.
    
    py_script = os.path.join("benchmark", "nptdms", "benchmark.py")
    py_args = [
        sys.executable, py_script,
        "--json",
        "--file", config['paths']['tdms_file'],
        "--samples", str(config['samples']),
        "--iterations", str(config['iterations']),
        "--warmup", str(config.get('warmup_iterations', 1))
    ]
    
    res = run_command(py_args)
    if res.returncode != 0:
        print(f"Python benchmark failed: {res.stderr}")
        py_res = None
    else:
        try:
            py_res = json.loads(res.stdout)
            print(f"Write: {py_res['write_gb_s']:.2f} GB/s")
            print(f"Read:  {py_res['read_gb_s']:.2f} GB/s")
        except Exception as e:
            print(f"Failed to parse Python output: {e}")
            print(res.stdout)
            py_res = None

    # 4. Generate Report
    generate_report(config, disk_write, disk_read, rust_res, py_res)
    
    # Cleanup data files if needed?
    # User said "TDMS files are generated per run and deleted (or reused)". 
    # Logic in scripts handles deletion/creation. 
    # We might want to remove the final file.
    if os.path.exists(config['paths']['tdms_file']):
        try:
            os.remove(config['paths']['tdms_file'])
        except:
            pass

def generate_report(config, disk_write, disk_read, rust_res, py_res):
    print_header("BENCHMARK REPORT")
    
    paths = config['paths']
    summary_path = os.path.join(paths['results_dir'], "summary.md")
    json_path = os.path.join(paths['results_dir'], "results.json")
    
    rust_w = rust_res['write_gb_s'] if rust_res else None
    rust_r = rust_res['read_gb_s'] if rust_res else None
    py_w = py_res['write_gb_s'] if py_res else None
    py_r = py_res['read_gb_s'] if py_res else None
    
    def pct(val, base):
        if val is None or base is None or base == 0: return "N/A"
        return f"{(val/base)*100:.1f}%"
    
    def fmt(val):
        return f"{val:.2f}" if val is not None else "N/A"

    # Console Table
    print(f"{'Operation':<10} | {'Disk (GB/s)':<12} | {'nptdms':<15} | {'tdms-rs':<15}")
    print("-" * 60)
    print(f"{'Write':<10} | {fmt(disk_write):<12} | {fmt(py_w)} ({pct(py_w, disk_write)}) | {fmt(rust_w)} ({pct(rust_w, disk_write)})")
    print(f"{'Read':<10} | {fmt(disk_read):<12} | {fmt(py_r)} ({pct(py_r, disk_read)}) | {fmt(rust_r)} ({pct(rust_r, disk_read)})")
    
    # JSON Output
    results = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "config": config,
        "disk": {"write_gb_s": disk_write, "read_gb_s": disk_read},
        "nptdms": py_res,
        "tdms_rs": rust_res
    }
    
    with open(json_path, "w") as f:
        json.dump(results, f, indent=2)
        
    print(f"\nJSON results saved to {json_path}")
    
    # Markdown Output
    md = f"""# TDMS Benchmark Results
    
**Date:** {results['timestamp']}
**System:** {platform.system()} {platform.release()} {platform.processor()}
**File Size:** {config['file_size_gb']} GB

| Operation | Disk (GB/s) | nptdms (GB/s) | nptdms % | tdms-rs (GB/s) | tdms-rs % |
|-----------|-------------|---------------|----------|----------------|-----------|
| Write     | {fmt(disk_write)} | {fmt(py_w)} | {pct(py_w, disk_write)} | {fmt(rust_w)} | {pct(rust_w, disk_write)} |
| Read      | {fmt(disk_read)}  | {fmt(py_r)} | {pct(py_r, disk_read)}  | {fmt(rust_r)} | {pct(rust_r, disk_read)}  |

## Methodology
- **Disk:** Sequential I/O using `diskspd` (Windows) or equivalent (Linux). Cache disabled.
- **nptdms:** `numpy` array write/read.
- **tdms-rs:** Rust `TdmsWriter`/`TdmsFile` generic write/read.
- **Baseline:** Disk speed is 100%.
"""
    with open(summary_path, "w") as f:
        f.write(md)
        
    print(f"Summary saved to {summary_path}")

if __name__ == "__main__":
    main()
