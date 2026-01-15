//! Tests based on the API design examples in api_design/

use tdms_rs::{TdmsFile, TdmsWriter};
use std::sync::Arc;
use std::thread;

#[test]
fn test_basic_read() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a data.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("data.tdms").exists() {
        return Ok(());
    }

    let f = TdmsFile::open("data.tdms")?;
    let ch = f.group("G").ok_or("group not found")?.channel("C").ok_or("channel not found")?;

    let slice = ch.read(0..100)?;
    let data: &[f64] = slice.as_typed()?;

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
    let ch = f.group("G").ok_or("group not found")?.channel("C").ok_or("channel not found")?;

    for chunk in ch.chunks(10_000_000) {
        let slice = chunk?;
        println!("chunk len = {}", slice.len());
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

    let handles: Vec<_> = (0..4).map(|i| {
        let f = f.clone();
        thread::spawn(move || {
            let ch = f.group("G").unwrap().channel("C").unwrap();
            let slice = ch.read(i*1_000_000..(i+1)*1_000_000).unwrap();
            slice.len()
        })
    }).collect();

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
    let ch = f.group("DAQ").ok_or("group not found")?.channel("Voltage").ok_or("channel not found")?;

    if let Some(ts) = ch.timestamps() {
        let slice = ch.read(0..ch.len())?;
        let values: &[f64] = slice.as_typed()?;

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

#[test]
fn test_zero_copy_check() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a data.tdms file
    // For now, we'll skip if the file doesn't exist
    if !std::path::Path::new("data.tdms").exists() {
        return Ok(());
    }

    let f = TdmsFile::open("data.tdms")?;
    let ch = f.group("G").ok_or("group not found")?.channel("C").ok_or("channel not found")?;

    let slice = ch.read(0..1000)?;
    println!("zero-copy: {}", slice.is_zero_copy());

    Ok(())
}
