use tdms_rs::{TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "large_data.tdms";

    // 1. Create a sample TDMS file with multiple segments/large data
    {
        let mut writer = TdmsWriter::create(path)?;
        let mut group = writer.add_group("G")?;
        let mut channel = group.add_channel::<f64>("C")?;

        // Write 150,000 samples to ensure we have enough to demonstrate chunking
        let data = vec![1.23; 50_000];
        channel.write(&data)?;
        channel.write(&data)?;
        channel.write(&data)?;

        writer.close()?;
    }

    // 2. Open the file for reading
    let file = TdmsFile::open(path)?;
    let channel = file
        .group("G")
        .ok_or("Group G not found")?
        .channel("C")
        .ok_or("Channel C not found")?;

    // 3. Process in chunks to keep memory usage constant regardless of file size
    let chunk_size = 50_000;
    let total_len = channel.len();
    let mut buffer = vec![0.0f64; chunk_size];

    println!(
        "Reading {} samples in chunks of {}...",
        total_len, chunk_size
    );

    for start in (0..total_len).step_by(chunk_size) {
        let end = (start + chunk_size).min(total_len);
        let count = end - start;

        channel.read(start..end, &mut buffer[0..count])?;
        let _data = &buffer[0..count];

        println!("  Read chunk: {}-{}", start, end);

        // Process data...
        // let _sum: f64 = data.iter().sum();
    }

    // Clean up
    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}
