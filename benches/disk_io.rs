//! # tdms-rs Disk I/O Benchmark
//!
//! This benchmark measures the **raw sequential throughput** of `tdms-rs` when reading and writing
//! large files (1.0 GB) to disk. It uses strict timing controls to exclude data generation and
//! memory copying where possible, focusing on the library's serialization overhead and disk I/O.
//!
//! ## Methodology
//!
//! - **Minimum Time**: Throughput is calculated using the *minimum* observed time across iterations.
//!   This represents the "achievable throughput ceiling" of the system, filtering out transient
//!   OS interruptions or background tasks.
//! - **Sequential Access**: Large blocks of data are written/read sequentially, matching the
//!   optimal access pattern for high-performance I/O.
//! - **Raw Disk Comparison**: The results should be compared against `diskspd` (Windows) or `dd` (Linux)
//!   benchmarks with similar parameters (1GB file, sequential, large buffers).
//!
//! ## Cache Behavior
//!
//! - **Write**: Uses `fsync` (via `writer.write()`) to ensure data is flushed to persistent storage.
//! - **Read**: Attempts to evict OS page cache between runs by allocating a large dummy buffer (2 GB).
//!   Note that this is a "best effort" approximation of cold reads; true cold reads require
//!   OS-level cache dropping which is not portable or guaranteed here.
//!
//! ## Interpretation
//!
//! - **Write (GB/s)**: Measures `tdms-rs` serialization + `std::fs::File` write + OS flush.
//! - **Read (GB/s)**: Measures `tdms-rs` parsing + `std::fs::File` read + memory allocation.
//!
//! Run with: `cargo bench --bench disk_io`

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tdms_rs::{TdmsData, TdmsFile, TdmsFileWriter};

// --- CONFIGURATION ---
const FILE_NAME: &str = "benchmark_disk_io.tdms";
const SAMPLE_COUNT: usize = 125_000_000; // 125M f64s * 8 bytes = 1.0 GB
const EXPECTED_SIZE_GB: f64 = (SAMPLE_COUNT * 8) as f64 / 1_000_000_000.0;
const WARMUP_SECS: u64 = 2;
const MEASURE_SECS: u64 = 30; // 30s for disk I/O is reasonable
const TARGET_SAMPLE_SIZE: usize = 10;

fn benchmark_disk_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("tdms_disk_io");
    group.sample_size(TARGET_SAMPLE_SIZE);
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(WARMUP_SECS));
    group.measurement_time(Duration::from_secs(MEASURE_SECS));

    println!("Generating {} samples ({} GB)...", SAMPLE_COUNT, EXPECTED_SIZE_GB);
    let data: Vec<f64> = (0..SAMPLE_COUNT).map(|i| i as f64).collect();

    group.bench_function("tdms_rs_write_full_channel_disk", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut min_time = f64::INFINITY;

            for _i in 0..iters {
                // Setup: Delete file to ensure fresh creation
                if Path::new(FILE_NAME).exists() {
                    let _ = fs::remove_file(FILE_NAME);
                }
                
                // Preparation: Clone data *outside* the timing window.
                // We want to measure I/O and library overhead, not Vec::clone().
                let data_clone = data.clone();

                // --- MEASUREMENT START ---
                let start = Instant::now();

                // 1. Create Writer
                let mut writer = TdmsFileWriter::new(FILE_NAME);
                
                // 2. Add Group & Channel (Data ownership transferred here)
                // Note: unwrap() is acceptable in bench harness panic
                let group = writer.add_group("BenchmarkGroup").unwrap();
                group.add_channel("BenchmarkChannel", TdmsData::Double(data_clone)).unwrap();
                
                // 3. Write & Flush (fsync)
                writer.write().unwrap();

                let elapsed = start.elapsed();
                // --- MEASUREMENT END ---

                total_duration += elapsed;
                
                let secs = elapsed.as_secs_f64();
                if secs < min_time {
                    min_time = secs;
                }

                // Cleanup logic implies file exists for next iteration's delete
            }
            
            // Print Report immediately for visibility
            let throughput = EXPECTED_SIZE_GB / min_time;
            println!(
                "\n[WRITE] Min Time: {:.4} s | Throughput: {:.2} GB/s", 
                min_time, throughput
            );
            
            total_duration
        })
    });
    group.finish();
}

fn benchmark_disk_read(c: &mut Criterion) {
    // Ensure file exists for read bench
    if !Path::new(FILE_NAME).exists() {
        println!("Creating file for read benchmark...");
        let data: Vec<f64> = (0..SAMPLE_COUNT).map(|i| i as f64).collect();
        let mut writer = TdmsFileWriter::new(FILE_NAME);
        let group = writer.add_group("BenchmarkGroup").unwrap();
        group.add_channel("BenchmarkChannel", TdmsData::Double(data)).unwrap();
        writer.write().unwrap();
    }

    let mut group = c.benchmark_group("tdms_disk_io");
    group.sample_size(TARGET_SAMPLE_SIZE);
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(WARMUP_SECS));
    group.measurement_time(Duration::from_secs(MEASURE_SECS));

    group.bench_function("tdms_rs_read_full_channel_disk", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut min_time = f64::INFINITY;

            for _ in 0..iters {
                // Best-effort Cache Eviction: Allocate 2x file size
                // This forces OS to swap out file pages if memory pressure is high
                let _dummy: Vec<u8> = vec![0u8; (SAMPLE_COUNT * 8) * 2]; 
                black_box(_dummy.as_ptr());
                drop(_dummy); // Ensure it's freed before measurement to reduce alloc pressure during bench?
                // Actually if we free it, OS might reclaim memory. Ideally we want to trash cache.
                // Allocation + writing to it trashes cache. `vec![0u8; size]` writes zeros.
                
                // --- MEASUREMENT START ---
                let start = Instant::now();

                // 1. Open File (Parses metadata)
                let file = TdmsFile::load(Path::new(FILE_NAME)).unwrap();
                
                // 2. Access Data (Materializes full channel into memory)
                if let Some(channel) = file.get_channel("BenchmarkGroup", "BenchmarkChannel") {
                    if let Some(data) = channel.as_f64() {
                        // 3. Touch data to prevent code elimination
                        black_box(data.len());
                    }
                }

                let elapsed = start.elapsed();
                // --- MEASUREMENT END ---

                total_duration += elapsed;

                let secs = elapsed.as_secs_f64();
                if secs < min_time {
                    min_time = secs;
                }
            }

            // Print Report
            let throughput = EXPECTED_SIZE_GB / min_time;
            println!(
                "\n[READ]  Min Time: {:.4} s | Throughput: {:.2} GB/s", 
                min_time, throughput
            );

            total_duration
        })
    });
    group.finish();
    
    // Final cleanup
    if Path::new(FILE_NAME).exists() {
        let _ = fs::remove_file(FILE_NAME);
    }
}

criterion_group!(benches, benchmark_disk_write, benchmark_disk_read);
criterion_main!(benches);
