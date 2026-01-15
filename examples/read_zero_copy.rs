use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::open("large_data.tdms")?;
    let channel = file
        .group("G")
        .ok_or("Group G not found")?
        .channel("C")
        .ok_or("Channel C not found")?;

    // THE ZERO-COPY PHILOSOPHY:
    // 1. TdmsFile::open() only reads metadata.
    // 2. channel.read() returns a TdmsSlice which may point to memory-mapped data.
    // 3. slice.as_typed() provides a direct &'a [T] reference to that data.

    // Process in chunks to keep memory usage constant regardless of file size
    let chunk_size = 100_000;
    for chunk in channel.chunks(chunk_size) {
        let slice = chunk?;

        // This is zero-copy if the platform supports it
        println!("Is zero-copy? {}", slice.is_zero_copy());

        let data: &[f64] = slice.as_typed()?;
        // Process data...
        let _sum: f64 = data.iter().sum();
    }

    Ok(())
}
