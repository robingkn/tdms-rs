use tdms::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = TdmsFile::open("daq.tdms")?;
    let ch = f.group("DAQ")?.channel("Voltage")?;

    if let Some(ts) = ch.timestamps() {
        let values = ch.read(0..ch.len())?.as_typed::<f64>()?;

        for (t, v) in ts.zip(values.iter()) {
            println!("{t} -> {v}");
        }
    }
    Ok(())
}
