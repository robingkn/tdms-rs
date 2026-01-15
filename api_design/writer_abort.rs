use tdms::TdmsWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut w = TdmsWriter::create("partial.tdms")?;

    let res: Result<(), Box<dyn std::error::Error>> = (|| {
        let mut g = w.add_group("G")?;
        let mut ch = g.add_channel::<f64>("C")?;
        ch.write(&[1.0, 2.0])?;
        Err("failure".into())
    })();

    if res.is_err() {
        w.abort()?;
    }
    Ok(())
}
