//! Tests based on the API design examples in api_design/

use std::sync::Arc;
use std::thread;
use tdms_rs::{TdmsFile, TdmsWriter};

#[test]
fn test_basic_read() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a data.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("data.tdms").exists() {
        return Ok(());
    }

    let f = TdmsFile::open("data.tdms")?;
    let ch = f
        .group("G")
        .ok_or("group not found")?
        .channel("C")
        .ok_or("channel not found")?;

    let mut data = vec![0.0f64; 100];
    ch.read_into(0..100, &mut data)?;

    println!("read {} values", data.len());
    Ok(())
}

#[test]
fn test_chunk_streaming() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a 1TB.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("1TB.tdms").exists() {
        return Ok(());
    }

    let f = TdmsFile::open("1TB.tdms")?;
    let ch = f
        .group("G")
        .ok_or("group not found")?
        .channel("C")
        .ok_or("channel not found")?;

    let chunk_size = 10_000_000;
    let total_len = ch.len();
    let mut buffer = vec![0.0f64; chunk_size];

    for start in (0..total_len).step_by(chunk_size) {
        let end = (start + chunk_size).min(total_len);
        let count = end - start;
        ch.read_into(start..end, &mut buffer[0..count])?;
        println!("chunk len = {}", count);
    }
    Ok(())
}

#[test]
fn test_inspect_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires an experiment.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("experiment.tdms").exists() {
        return Ok(());
    }

    let f = TdmsFile::open("experiment.tdms")?;

    for g in f.groups() {
        println!("Group: {}", g.name());

        for ch in g.channels() {
            println!("  Channel: {}", ch.name());
            println!("    dtype  = {:?}", ch.dtype());
            println!("    len    = {}", ch.len());
        }
    }
    Ok(())
}

#[test]
fn test_parallel_reads() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a big.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("big.tdms").exists() {
        return Ok(());
    }

    let f = Arc::new(TdmsFile::open("big.tdms")?);

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let f = f.clone();
            thread::spawn(move || {
                let ch = f.group("G").unwrap().channel("C").unwrap();
                let mut data = vec![0.0f64; 1_000_000];
                ch.read_into(i * 1_000_000..(i + 1) * 1_000_000, &mut data)
                    .unwrap();
                data.len()
            })
        })
        .collect();

    for h in handles {
        println!("read {}", h.join().unwrap());
    }
    Ok(())
}

#[test]
fn test_timestamps() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a daq.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("daq.tdms").exists() {
        return Ok(());
    }

    let f = TdmsFile::open("daq.tdms")?;
    let ch = f
        .group("DAQ")
        .ok_or("group not found")?
        .channel("Voltage")
        .ok_or("channel not found")?;

    if let Some(ts) = ch.timestamps() {
        let mut values = vec![0.0f64; ch.len()];
        ch.read_into(0..ch.len(), &mut values)?;

        for (t, v) in ts.zip(values.iter()) {
            println!("{t} -> {v}");
        }
    }
    Ok(())
}

#[test]
fn test_writer_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut w = TdmsWriter::create("test_out.tdms")?;
    let mut g = w.add_group("DAQ")?;
    let mut ch = g.add_channel::<f64>("Voltage")?;

    ch.write(&[1.0, 2.0, 3.0])?;
    w.close()?;

    // Clean up
    std::fs::remove_file("test_out.tdms").ok();
    Ok(())
}

#[test]
fn test_writer_abort() -> Result<(), Box<dyn std::error::Error>> {
    let mut w = TdmsWriter::create("partial.tdms")?;

    let res: Result<(), Box<dyn std::error::Error>> = (|| {
        let mut g = w.add_group("G")?;
        let mut ch = g.add_channel::<f64>("C")?;
        ch.write(&[1.0, 2.0])?;
        Err("failure".into())
    })();

    if res.is_err() {
        w.abort()?;
    }

    // Verify file was deleted
    assert!(!std::path::Path::new("partial.tdms").exists());
    Ok(())
}
