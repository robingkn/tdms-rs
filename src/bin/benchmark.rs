use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use serde::Serialize;
use tdms_rs::{TdmsData, TdmsFile, TdmsFileWriter};

// Configuration
// 125M f64s * 8 bytes = 1.0 GB strictly
const SAMPLE_COUNT: usize = 125_000_000; 
const FILE_NAME: &str = "benchmark_rust.tdms";
const WARMUP_RUNS: usize = 1;
const MEASURED_RUNS: usize = 5;

#[derive(Serialize)]
struct BenchmarkResult {
    write_gb_s: f64,
    write_min_time: f64,
    read_gb_s: f64,
    read_min_time: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let json_mode = args.contains(&"--json".to_string());

    if !json_mode {
        println!("=== tdms-rs Full-Channel Benchmark ===");
        println!("File Name: {}", FILE_NAME);
    }

    let data_size_bytes = SAMPLE_COUNT * 8;
    // We follow the 10^9 convention for GB as decided in the plan
    let data_size_gb = data_size_bytes as f64 / 1_000_000_000.0;

    if !json_mode {
        println!("Data Size: {:.4} GB ({} samples)", data_size_gb, SAMPLE_COUNT);
        println!("\n--- Write Benchmark ---");
    }
    
    let (write_speed, write_time) = run_write_benchmark(data_size_gb, json_mode)?;
    
    if !json_mode {
        println!("\n--- Read Benchmark ---");
    }
    
    let (read_speed, read_time) = run_read_benchmark(data_size_gb, json_mode)?;
    
    if json_mode {
        let result = BenchmarkResult {
            write_gb_s: write_speed,
            write_min_time: write_time,
            read_gb_s: read_speed,
            read_min_time: read_time,
        };
        let json_str = serde_json::to_string(&result)?;
        println!("{}", json_str);
    } else {
        println!("\n=== SUMMARY ===");
        println!("tdms-rs Write: {:.2} GB/s (min time: {:.4}s)", write_speed, write_time);
        println!("tdms-rs Read:  {:.2} GB/s (min time: {:.4}s)", read_speed, read_time);
    }
    
    // Cleanup
    if Path::new(FILE_NAME).exists() {
        fs::remove_file(FILE_NAME)?;
    }

    Ok(())
}

fn run_write_benchmark(size_gb: f64, silent: bool) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    if !silent {
        println!("Generating {} samples of f64 data...", SAMPLE_COUNT);
    }
    
    // Generate data once
    let data: Vec<f64> = (0..SAMPLE_COUNT).map(|i| i as f64).collect();
    let mut times = Vec::new();

    for i in 0..(WARMUP_RUNS + MEASURED_RUNS) {
        let is_warmup = i < WARMUP_RUNS;
        if !silent {
            print!("Run {}: ", if is_warmup { "Warmup" } else { "Measure" });
            std::io::stdout().flush()?;
        }

        // Clean up previous file if exists to ensure cold create
        if Path::new(FILE_NAME).exists() {
            fs::remove_file(FILE_NAME)?;
        }

        let start = Instant::now();
        
        let mut writer = TdmsFileWriter::new(FILE_NAME);
        let group = writer.add_group("BenchmarkGroup")?;
        group.add_channel("BenchmarkChannel", TdmsData::Double(data.clone()))?;
        writer.write()?;
        
        // sync_all is usually what TdmsFileWriter calls internally or we trust std::fs to flush on close/drop?
        // tdms-rs `write()` calls `flush()`. To be extra safe we could try to sync directory, but simplest is trust write().
        
        let duration = start.elapsed();
        let seconds = duration.as_secs_f64();
        let speed = size_gb / seconds;
        
        if !silent {
            println!("{:.4} s ({:.2} GB/s)", seconds, speed);
        }

        if !is_warmup {
            times.push(seconds);
        }
    }

    let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_throughput = size_gb / min_time;
    Ok((max_throughput, min_time))
}

fn run_read_benchmark(size_gb: f64, silent: bool) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    // Ensure file exists (re-create if needed from write bench, but write bench should have left it? 
    // Ah, write bench deletes it at the start of loop. The last run of write bench leaves the file?
    // In write bench loop: "Clean up previous file if exists". 
    // So after the loop finishes, the file FROM THE LAST RUN exists.
    // BUT, wait.
    // Loop `for i in ...`:
    //   delete file
    //   write file
    // loop ends.
    // So the file exists.
    
    if !Path::new(FILE_NAME).exists() {
        return Err("Benchmark file not found for read test".into());
    }

    let mut times = Vec::new();

    for i in 0..(WARMUP_RUNS + MEASURED_RUNS) {
        let is_warmup = i < WARMUP_RUNS;
        if !silent {
            print!("Run {}: ", if is_warmup { "Warmup" } else { "Measure" });
            std::io::stdout().flush()?;
        }

        let start = Instant::now();
        
        let path = Path::new(FILE_NAME);
        let file = TdmsFile::load(path)?;
        
        if let Some(channel) = file.get_channel("BenchmarkGroup", "BenchmarkChannel") {
            if let Some(read_data) = channel.as_f64() {
                // Force evaluation
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
        
        if !silent {
            println!("{:.4} s ({:.2} GB/s)", seconds, speed);
        }

        if !is_warmup {
            times.push(seconds);
        }
    }

    let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_throughput = size_gb / min_time;
    Ok((max_throughput, min_time))
}
