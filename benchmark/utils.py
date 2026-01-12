import numpy as np
import time

def clobber_cache(file_size_gb):
    """
    Standardized memory clobbering to evict OS page cache.
    Allocates 4x file size, touches each 4KB page, and drops.
    """
    clobber_size_gb = file_size_gb * 4
    # 1 GB = 10^9 bytes for consistency
    n_bytes = int(clobber_size_gb * 1e9)
    # n_items = n_bytes (using uint8)
    
    print(f"[INFO] Clobbering cache: allocating {clobber_size_gb:.1f} GB...")
    
    # Allocate
    try:
        # Step through indices to touch each 4096-byte page
        # Using numpy for efficient allocation but slow-ish touching loop
        buf = np.zeros(n_bytes, dtype=np.uint8)
        
        # Touch 1 byte per 4KB page
        indices = np.arange(0, n_bytes, 4096)
        buf[indices] += 1
        
        # Black box equivalent: ensure sum is used
        _ = np.sum(buf[0:100])
        
        del buf
    except MemoryError:
        print("[WARNING] Could not allocate clobber buffer. Run might be warm.")
