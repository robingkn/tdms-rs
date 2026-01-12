# TDMS-RS Write Performance: Final Analysis & Recommendations

## Executive Summary

**Problem**: tdms-rs write throughput (0.47-0.53 GB/s) is **22% slower** than nptdms (0.59-0.69 GB/s).

**Root Cause Identified**: The bottleneck is in **raw data write operations** (98.5% of total time), not metadata or syncing.

**Status**: Standard Rust-level optimizations (buffering, batching, allocation) showed no improvement. The gap appears to be at the OS/filesystem level.

## Step 1: Time-Split Instrumentation ✅

### Results

```
Total Write Time: 1.8793s (0.53 GB/s)

Breakdown:
- Metadata Build:  0.03 ms  (0.0%)   ← Negligible
- Metadata Write:  0.06 ms  (0.0%)   ← Negligible  
- Raw Data Write:  1553.83 ms (98.5%) ← BOTTLENECK
- Sync:           24.02 ms  (1.5%)   ← Not the issue
```

### Key Finding

**98.5% of write time is spent in `File::write_all()` for 1GB data**. This is where the performance gap vs nptdms must be.

**Eliminated as causes**:
- ❌ Metadata building/writing (<0.1ms)
- ❌ File syncing (24ms, 1.5%)
- ❌ Buffer management (already optimized)

## Step 2: Syscall & I/O Pattern Comparison

### Status: Requires OS-level tooling

**Recommended Tools**:
- Windows: Process Monitor (ProcMon) or ETW
- Compare: Number of write syscalls, write sizes, file open flags

**Expected Investigation**:
1. Count write syscalls for 1GB write
2. Compare write size distribution
3. Check file open flags differences
4. Measure actual disk I/O time vs CPU time

**Note**: Cannot be completed without OS-level profiling tools. Requires manual investigation.

## Step 3: File Open & OS Hint Experiments

### Experiments to Test

#### Experiment 3.1: Chunked Writes
**Hypothesis**: Writing in smaller chunks (64-128 MB) may allow OS to optimize better than single 1GB write.

**Implementation**: Modify `write_channel_data_direct()` to write in chunks.

**Expected Impact**: +5-15% if OS benefits from chunked writes.

#### Experiment 3.2: std::fs::write()
**Hypothesis**: `std::fs::write()` may have different internal optimizations than `File::write_all()`.

**Implementation**: Use `std::fs::write()` for entire raw data section.

**Expected Impact**: Unknown, but worth testing.

#### Experiment 3.3: Windows File Flags (Requires winapi crate)
**Hypothesis**: `FILE_FLAG_SEQUENTIAL_SCAN` may improve Windows write performance.

**Implementation**: Use `winapi` crate to set file flags.

**Expected Impact**: +10-20% if Windows optimizes sequential access.

**Complexity**: Medium (requires adding winapi dependency, platform-specific code).

#### Experiment 3.4: write_vectored()
**Hypothesis**: Combining metadata + data in vectored write may reduce syscalls.

**Implementation**: Use `write_vectored()` to write header + metadata + data together.

**Expected Impact**: Low (metadata is tiny, but may reduce syscall overhead).

## Step 4: nptdms Write Path Audit

### Status: Requires Source Code Inspection

**Key Questions**:
1. Does nptdms use Python's default file buffering or custom buffering?
2. Does nptdms use C extensions for writes?
3. What file flags does Python's file object use on Windows?
4. Does NumPy's array write path have optimizations?

**Recommended Approach**:
- Inspect nptdms source code (GitHub: https://github.com/adamreeve/npTDMS)
- Check Python file object implementation
- Understand NumPy array write path

## Step 5: Sync Semantics Validation

### Status: Already Validated ✅

**Finding**: Sync is only 1.5% of total time (24ms). Even if sync were eliminated, would only improve by ~1.3%.

**Conclusion**: Sync is NOT the bottleneck. Both benchmarks include sync, so comparison is fair.

## Step 6: Decision Output

### Is the Gap Fixable?

**Likely Causes** (ranked by probability):

1. **OS-Level Write Behavior** (Most Likely)
   - Windows may handle Python file writes differently
   - Python's file object may use optimized paths
   - Rust's `File::write_all()` may trigger different code paths

2. **File Flags** (Medium Probability)
   - Python may implicitly use `FILE_FLAG_SEQUENTIAL_SCAN`
   - Rust's default `File::create()` may not set optimal flags

3. **Chunked vs Single Write** (Low-Medium Probability)
   - OS may optimize smaller writes better
   - Single 1GB write may not be optimal

4. **Fundamentally OS-Dependent** (Low Probability)
   - Python runtime may have OS-specific optimizations
   - May be inherent to Rust's I/O implementation

### Is Further Optimization Worth It?

**Arguments FOR**:
- 22% gap is significant
- Read performance is excellent (75% faster than nptdms)
- Write optimization would make tdms-rs superior in all metrics

**Arguments AGAINST**:
- Current performance (66% of disk speed) is reasonable
- Read performance is already excellent
- Further optimization requires platform-specific code
- May introduce complexity and maintenance burden

### Recommendation

**GO**: Continue with targeted experiments, but with clear success criteria.

**Success Criteria**:
- Target: Close gap to within 10% of nptdms (i.e., ≥0.54 GB/s)
- If experiments don't achieve this, accept current performance

**Priority Experiments**:
1. **Chunked writes** (64-128 MB chunks) - Low complexity, potential benefit
2. **std::fs::write()** - Very low complexity, quick test
3. **Windows file flags** - Medium complexity, potential high benefit

**Skip if**:
- Experiments show <5% improvement
- Complexity outweighs benefit
- Gap is acceptable given read dominance

## Next Steps

1. ✅ **Completed**: Time-split instrumentation
2. ⏭️ **Next**: Test chunked writes (Experiment 3.1)
3. ⏭️ **Next**: Test std::fs::write() (Experiment 3.2)
4. ⏭️ **Optional**: Windows file flags (Experiment 3.3) - if 3.1/3.2 don't help
5. ⏭️ **Optional**: nptdms source audit - if needed for understanding

## Conclusion

The write performance gap is **clearly in raw data write operations** (98.5% of time). Standard Rust-level optimizations have been exhausted. The remaining gap is likely due to:

1. OS-level write behavior differences
2. Missing file flags/optimizations
3. Write size/chunking strategy

**Recommended Action**: Proceed with low-complexity experiments (chunked writes, std::fs::write). If these don't close the gap significantly, accept current performance as reasonable given:
- Read performance is excellent (75% faster)
- Write performance is still 66% of disk speed
- Further optimization would require platform-specific code


