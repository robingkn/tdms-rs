use tdms::{TdmsFile, ChannelData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = TdmsFile::open("data.tdms")?;
    let ch = f.group("G")?.channel("C")?;

    let slice = ch.read(0..1000)?;
    println!("zero-copy: {}", slice.is_zero_copy());

    Ok(())
}
