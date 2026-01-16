use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::open("large_data.tdms")?;
    let channel = file
        .group("G")
        .ok_or("Group G not found")?
        .channel("C")
        .ok_or("Channel C not found")?;

    // Process in chunks to keep memory usage constant regardless of file size
    let chunk_size = 100_000;
    let total_len = channel.len();
    let mut buffer = vec![0.0f64; chunk_size];

    for start in (0..total_len).step_by(chunk_size) {
        let end = (start + chunk_size).min(total_len);
        let count = end - start;

        channel.read_into(start..end, &mut buffer[0..count])?;
        let data = &buffer[0..count];

        // Process data...
        let _sum: f64 = data.iter().sum();
    }

    Ok(())
}
