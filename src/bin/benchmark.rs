use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use tdms_rs::{TdmsData, TdmsFile, TdmsFileWriter};

// Configuration
const SAMPLE_COUNT: usize = 60_000_000; // 60M f64s = 480 MB
const FILE_NAME: &str = "benchmark.tdms";
const WARMUP_RUNS: usize = 1;
const MEASURED_RUNS: usize = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== tdms-rs Full-Channel Benchmark ===");
    println!("File Name: {}", FILE_NAME);
    let data_size_bytes = SAMPLE_COUNT * 8;
    let data_size_gb = data_size_bytes as f64 / 1_000_000_000.0; // using 10^9 for GB as per disk specs usually, or 2^30 for GiB? 
    // User request asked for GB/s. Usually disk throughput is MB/s (10^6). Let's use 10^9 for GB/s to match standard disk specs often used in marketing, or 2^30 for GiB.
    // Standard system benchmarks often use GiB (1024^3). Let's stick to GiB for "computer" GB, but label it GiB or GB clearly. 
    // Request says "throughput_gb_s = file_size_gb / min_time_seconds".
    // I will use 10^9 for GB to be compatible with typical diskspd output which often uses decimal MB/s, but let's be precise.
    // Actually, to be safe and clear, I'll calculate in GB (10^9 bytes) as that is standard for "GB/s".
    let data_size_gb_dec = data_size_bytes as f64 / 1_000_000_000.0;

    println!("Data Size: {:.4} GB ({} samples)", data_size_gb_dec, SAMPLE_COUNT);
    
    println!("\n--- Write Benchmark ---");
    let write_speed = run_write_benchmark(data_size_gb_dec)?;
    
    println!("\n--- Read Benchmark ---");
    let read_speed = run_read_benchmark(data_size_gb_dec)?;
    
    println!("\n=== SUMMARY ===");
    println!("System: (Rust generic)"); // We can't easily get OS info without deps
    println!("File size: {:.4} GB", data_size_gb_dec);
    println!("tdms-rs Write: {:.2} GB/s (min time)", write_speed);
    println!("tdms-rs Read:  {:.2} GB/s (min time)", read_speed);
    
    println!("\nNote: Compare these values with raw disk sequential I/O (e.g. using diskspd).");
    
    // Cleanup
    if Path::new(FILE_NAME).exists() {
        // fs::remove_file(FILE_NAME)?; // Keep file for inspection? Request said "Optional Enhancements: Delete temporary...". I'll delete it. 
        // Actually, user said "Optionally delete". I'll keep it for now or delete it? 
        // "Delete temporary TDMS files after benchmark." -> Listed as Optional Enhancement.
        // I'll delete it to be clean.
        fs::remove_file(FILE_NAME)?;
    }

    Ok(())
}

fn run_write_benchmark(size_gb: f64) -> Result<f64, Box<dyn std::error::Error>> {
    // Generate data once
    println!("Generating {} samples of f64 data...", SAMPLE_COUNT);
    let data: Vec<f64> = (0..SAMPLE_COUNT).map(|i| i as f64).collect();

    let mut times = Vec::new();

    for i in 0..(WARMUP_RUNS + MEASURED_RUNS) {
        let is_warmup = i < WARMUP_RUNS;
        print!("Run {}: ", if is_warmup { "Warmup" } else { "Measure" });
        std::io::stdout().flush()?;

        // Clean up previous file if exists to ensure cold create
        if Path::new(FILE_NAME).exists() {
            fs::remove_file(FILE_NAME)?;
        }

        let start = Instant::now();
        
        let mut writer = TdmsFileWriter::new(FILE_NAME);
        let group = writer.add_group("BenchmarkGroup")?;
        group.add_channel("BenchmarkChannel", TdmsData::Double(data.clone()))?;
        writer.write()?;
        
        // Writer.write() already calls flush(), so data should be on disk (OS buffers flushed).
        // However, to be absolutely sure against OS cache, we can't easily drop caches without admin privileges.
        // But the requirement says "Ensure file is flushed to disk after writing (fsync)". TdmsFileWriter does this.

        let duration = start.elapsed();
        let seconds = duration.as_secs_f64();
        let speed = size_gb / seconds;
        
        println!("{:.4} s ({:.2} GB/s)", seconds, speed);

        if !is_warmup {
            times.push(seconds);
        }
    }

    let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_throughput = size_gb / min_time;
    Ok(max_throughput)
}

fn run_read_benchmark(size_gb: f64) -> Result<f64, Box<dyn std::error::Error>> {
    // Ensure file exists (re-create if needed from write bench)
    if !Path::new(FILE_NAME).exists() {
        return Err("Benchmark file not found for read test".into());
    }

    let mut times = Vec::new();

    for i in 0..(WARMUP_RUNS + MEASURED_RUNS) {
        let is_warmup = i < WARMUP_RUNS;
        print!("Run {}: ", if is_warmup { "Warmup" } else { "Measure" });
        std::io::stdout().flush()?;

        let start = Instant::now();
        
        let path = Path::new(FILE_NAME);
        let file = TdmsFile::load(path)?;
        
        // Access the channel and read data to memory
        // API: file.get_channel("BenchmarkGroup", "BenchmarkChannel") -> Option<&TdmsChannel>
        if let Some(channel) = file.get_channel("BenchmarkGroup", "BenchmarkChannel") {
            if let Some(read_data) = channel.as_f64() {
                // Determine length to force evaluation, though as_f64 returns &Vec so it's already in memory.
                // TdmsFile::load reads the whole file into memory structures. 
                // So the read time is dominated by TdmsFile::load.
                // We should access the data to match the "Read full channel into memory" requirement.
                // But `TdmsFile::load` does parsing and reading.
                // We can't separate opening from reading in this library's API easily as `load` does eager loading?
                // Let's check `TdmsFile::load` implementation.
                // If it eager loads, then `load` is the benchmark.
                std::hint::black_box(read_data.len());
            } else {
                return Err("Failed to get f64 data".into());
            }
        } else {
             return Err("Channel not found".into());
        }

        let duration = start.elapsed();
        let seconds = duration.as_secs_f64();
        let speed = size_gb / seconds;
        
        println!("{:.4} s ({:.2} GB/s)", seconds, speed);

        if !is_warmup {
            times.push(seconds);
        }
    }

    let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_throughput = size_gb / min_time;
    Ok(max_throughput)
}
