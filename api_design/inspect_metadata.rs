use tdms::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
