use std::path::Path;
use std::time::Instant;
use tdms_rs::TdmsFile;

#[repr(C)]
#[allow(non_snake_case)]
struct PROCESS_MEMORY_COUNTERS {
    cb: u32,
    PageFaultCount: u32,
    PeakWorkingSetSize: usize,
    WorkingSetSize: usize,
    QuotaPeakPagedPoolUsage: usize,
    QuotaPagedPoolUsage: usize,
    QuotaPeakNonPagedPoolUsage: usize,
    QuotaNonPagedPoolUsage: usize,
    PagefileUsage: usize,
    PeakPagefileUsage: usize,
}

#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(
        process: *mut std::ffi::c_void,
        counters: *mut PROCESS_MEMORY_COUNTERS,
        cb: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
}

fn get_memory_usage_mb() -> f64 {
    let mut counters = PROCESS_MEMORY_COUNTERS {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        PageFaultCount: 0,
        PeakWorkingSetSize: 0,
        WorkingSetSize: 0,
        QuotaPeakPagedPoolUsage: 0,
        QuotaPagedPoolUsage: 0,
        QuotaPeakNonPagedPoolUsage: 0,
        QuotaNonPagedPoolUsage: 0,
        PagefileUsage: 0,
        PeakPagefileUsage: 0,
    };

    unsafe {
        let process = GetCurrentProcess();
        GetProcessMemoryInfo(process, &mut counters, counters.cb);
    }

    counters.WorkingSetSize as f64 / (1024.0 * 1024.0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "temp_load_test.tdms";
    if !Path::new(filename).exists() {
        println!("Please run the python loading benchmark first to generate '{}', or provide a valid TDMS file.", filename);
        return Ok(());
    }

    println!("--- Rust (tdms-rs) Loading Strategy ---");

    // 1. Initial State
    let mem_init = get_memory_usage_mb();
    println!("Initial Memory: {:.2} MB", mem_init);

    // 2. Load File (Eager)
    println!("\nLoading file (TdmsFile::load)...");
    let t0 = Instant::now();
    let file = TdmsFile::load(Path::new(filename))?;
    let t_load = t0.elapsed();
    
    let mem_after_load = get_memory_usage_mb();
    println!("Memory after TdmsFile::load(): {:.2} MB (Δ {:.2} MB)", mem_after_load, mem_after_load - mem_init);
    println!("Load Time: {:?}", t_load);

    // 3. Access Data (Already in memory)
    println!("\nAccessing Channel1 data...");
    let t1 = Instant::now();
    if let Some(channel) = file.get_channel("Group1", "Channel1") {
        if let Some(data) = channel.as_f64() {
             println!("Access Time: {:?}", t1.elapsed());
             println!("Data Samples: {}", data.len());
        }
    }

    Ok(())
}
