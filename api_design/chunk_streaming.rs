use tdms::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = TdmsFile::open("1TB.tdms")?;
    let ch = f.group("G")?.channel("C")?;

    for chunk in ch.chunks(10_000_000) {
        let slice = chunk?;
        println!("chunk len = {}", slice.len());
    }
    Ok(())
}
