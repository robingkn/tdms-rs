//! Demonstrates reading a large channel in fixed-size chunks to keep memory usage bounded.
//! Writes a large channel, then processes it in chunks without loading entire data into memory.

use std::path::Path;
use tdms_rs::{TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("chunked_processing.tdms");
    let total_samples = 150_000;
    let chunk_size = 50_000;

    // Write a large channel
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("Data")?;
        let mut channel = group.add_channel::<f64>("Signal")?;
        let data = vec![1.23; total_samples];
        channel.write(&data)?;
        // File is automatically flushed and closed when writer goes out of scope
    }

    // Read and process in chunks
    let file = TdmsFile::open(path)?;
    let channel = file.group("Data").unwrap().channel("Signal").unwrap();

    let total_len = channel.len();
    assert_eq!(total_len, total_samples);

    let mut buffer = vec![0.0f64; chunk_size];
    let mut processed = 0;

    println!(
        "Processing {} samples in chunks of {}...",
        total_len, chunk_size
    );

    for start in (0..total_len).step_by(chunk_size) {
        let end = (start + chunk_size).min(total_len);
        let count = end - start;

        channel.read(start..end, &mut buffer[0..count])?;
        let chunk = &buffer[0..count];

        // Example processing: compute sum for this chunk
        let sum: f64 = chunk.iter().sum();
        processed += count;

        println!("Chunk {}-{}: sum={:.2}", start, end, sum);
    }

    assert_eq!(processed, total_samples);
    println!("Processed {} total samples.", processed);

    // Clean up
    std::fs::remove_file(path)?;
    Ok(())
}
