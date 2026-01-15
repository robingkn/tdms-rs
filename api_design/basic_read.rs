use tdms::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = TdmsFile::open("data.tdms")?;
    let ch = f.group("G")?.channel("C")?;

    let slice = ch.read(0..100)?;
    let data: &[f64] = slice.as_typed()?;

    println!("read {} values", data.len());
    Ok(())
}
