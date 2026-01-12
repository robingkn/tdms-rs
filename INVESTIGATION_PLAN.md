# Write Performance Investigation Plan

## Completed Steps

### ✅ Step 1: Time-Split Instrumentation
**Result**: Identified bottleneck is raw data write (98.5% of time), not sync (1.5%)

## Remaining Steps

### Step 2: Syscall & I/O Pattern Comparison
**Goal**: Compare Rust vs nptdms at OS level

**Tools**:
- Windows: Process Monitor (ProcMon) or ETW
- Compare: Number of write syscalls, write sizes, file open flags

**Expected Output**:
- Syscall count comparison
- Write size distribution
- File flag differences

### Step 3: File Open & OS Hint Experiments
**Experiments**:
1. `FILE_FLAG_SEQUENTIAL_SCAN` on Windows
2. `FILE_FLAG_NO_BUFFERING` (with alignment safeguards)
3. `OpenOptions` vs `File::create`
4. `write_vectored` API
5. Chunked writes (64-128 MB) vs single 1GB write

**Each experiment**:
- Modify writer.rs
- Run benchmark
- Report delta vs baseline

### Step 4: nptdms Write Path Audit
**Goal**: Inspect nptdms source to understand optimizations

**Check**:
- Python file buffering strategy
- NumPy array write path
- C extension optimizations
- File flags used

### Step 5: Sync Semantics Validation
**Goal**: Verify sync equivalence

**Check**:
- Python `fsync()` vs Rust `sync_all()` behavior
- Whether nptdms benchmark includes deferred flush
- Re-benchmark with sync excluded (unsafe, for comparison)

### Step 6: Decision Output
**Deliverables**:
- Is gap fixable with OS hints/API changes?
- Is gap fundamentally OS-dependent?
- Is gap acceptable given read dominance?
- Go/no-go recommendation


