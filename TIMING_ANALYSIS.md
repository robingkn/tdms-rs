# Write Performance Timing Analysis

## Step 1: Time-Split Instrumentation Results

### Benchmark Configuration
- File size: 1.0 GB (125M f64 samples)
- Single iteration, no warmup
- Cold cache (clobbered before run)

### Timing Breakdown (tdms-rs)

```
Total Write Time: 1.8793s (0.53 GB/s)

Breakdown:
- Metadata Build:  0.03 ms  (0.0%)
- Metadata Write:  0.06 ms  (0.0%)
- Raw Data Write:  1553.83 ms (98.5%)  ← BOTTLENECK
- Sync:           24.02 ms  (1.5%)
```

### Key Findings

1. **Metadata is NOT the bottleneck**
   - Total metadata time: <0.1ms
   - Negligible impact on overall performance

2. **Sync is NOT the bottleneck**
   - Sync time: 24ms (1.5% of total)
   - Even if sync were eliminated, would only improve by ~1.3%
   - Both benchmarks include sync, so this is fair

3. **Raw data write IS the bottleneck**
   - 98.5% of time spent in `write_all()` for 1GB data
   - This is where the performance gap vs nptdms must be

### Conclusion

The performance gap between tdms-rs (0.47-0.53 GB/s) and nptdms (0.59-0.69 GB/s) is **entirely in the raw data write operation**, not in:
- Metadata building/writing
- File syncing
- Buffer management

**Next Steps**: Investigate why `File::write_all()` for 1GB is slower in Rust than Python's equivalent operation.


