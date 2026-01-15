use tdms::TdmsWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut w = TdmsWriter::create("out.tdms")?;
    let mut g = w.add_group("DAQ")?;
    let mut ch = g.add_channel::<f64>("Voltage")?;

    ch.write(&[1.0, 2.0, 3.0])?;
    w.close()?;
    Ok(())
}
