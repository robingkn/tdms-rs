import os
import time
import ctypes
from nptdms import TdmsFile, TdmsWriter, ChannelObject

def get_memory_usage_mb():
    """Returns the resident set size in MB using Windows API."""
    process_handle = ctypes.windll.kernel32.GetCurrentProcess()
    
    class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
        _fields_ = [
            ("cb", ctypes.c_ulong),
            ("PageFaultCount", ctypes.c_ulong),
            ("PeakWorkingSetSize", ctypes.c_size_t),
            ("WorkingSetSize", ctypes.c_size_t),
            ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
            ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
            ("PagefileUsage", ctypes.c_size_t),
            ("PeakPagefileUsage", ctypes.c_size_t),
        ]
    
    counters = PROCESS_MEMORY_COUNTERS()
    counters.cb = ctypes.sizeof(PROCESS_MEMORY_COUNTERS)
    ctypes.windll.psapi.GetProcessMemoryInfo(process_handle, ctypes.byref(counters), counters.cb)
    return counters.WorkingSetSize / (1024 * 1024)

def run_python_benchmark(filename):
    print(f"--- Python (nptdms) Loading Strategy ---")
    
    # 1. Initial State
    mem_init = get_memory_usage_mb()
    print(f"Initial Memory: {mem_init:.2f} MB")
    
    # 2. Open File (Metadata Load)
    t0 = time.perf_counter()
    with TdmsFile.read(filename) as tdms_file:
        t_open = time.perf_counter() - t0
        mem_after_open = get_memory_usage_mb()
        print(f"Memory after TdmsFile.read(): {mem_after_open:.2f} MB (Δ {mem_after_open - mem_init:.2f} MB)")
        print(f"Open Time: {t_open:.4f}s")
        
        # 3. Access Data
        print("\nAccessing Channel1 data...")
        t1 = time.perf_counter()
        channel = tdms_file["Group1"]["Channel1"]
        data = channel[:] # Force load
        t_access = time.perf_counter() - t1
        
        mem_after_access = get_memory_usage_mb()
        print(f"Memory after channel[:]: {mem_after_access:.2f} MB (Δ {mem_after_access - mem_after_open:.2f} MB)")
        print(f"Access Time: {t_access:.4f}s")
        print(f"Data Samples: {len(data)}")

if __name__ == "__main__":
    FILENAME = "temp_load_test.tdms"
    SAMPLE_COUNT = 50_000_000 # ~400MB
    
    if not os.path.exists(FILENAME):
        print(f"Generating test file {FILENAME}...")
        import numpy as np
        data = np.random.rand(SAMPLE_COUNT).astype(np.float64)
        with TdmsWriter(FILENAME) as writer:
            writer.write_segment([ChannelObject("Group1", "Channel1", data)])
        del data
        
    run_python_benchmark(FILENAME)
