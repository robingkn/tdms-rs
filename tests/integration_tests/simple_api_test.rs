//! Simple API test to verify the redesigned API works

use tdms_rs::{TdmsFile, TdmsWriter};

#[test]
fn test_write_and_read_simple() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test_simple.tdms";

    // Write
    {
        let mut w = TdmsWriter::create(test_file)?;
        let mut g = w.add_group("TestGroup")?;
        let mut ch = g.add_channel::<f64>("TestChannel")?;
        ch.write(&[1.0, 2.0, 3.0, 4.0, 5.0])?;
        w.close()?;
    }

    // Read
    {
        let f = TdmsFile::open(test_file)?;
        let g = f.group("TestGroup").unwrap();
        let ch = g.channel("TestChannel").unwrap();

        assert_eq!(ch.len(), 5);

        let slice = ch.read(0..5)?;
        let data: &[f64] = slice.as_typed()?;

        assert_eq!(data, &[1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    // Clean up
    std::fs::remove_file(test_file).ok();

    Ok(())
}

#[test]
fn test_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test_chunks.tdms";

    // Write
    {
        let mut w = TdmsWriter::create(test_file)?;
        let mut g = w.add_group("G")?;
        let mut ch = g.add_channel::<f64>("C")?;
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        ch.write(&data)?;
        w.close()?;
    }

    // Read in chunks
    {
        let f = TdmsFile::open(test_file)?;
        let ch = f.group("G").unwrap().channel("C").unwrap();

        let mut count = 0;
        for chunk in ch.chunks(25) {
            let slice = chunk?;
            assert!(slice.len() <= 25);
            count += slice.len();
        }

        assert_eq!(count, 100);
    }

    // Clean up
    std::fs::remove_file(test_file).ok();

    Ok(())
}

#[test]
fn test_writer_abort() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test_abort.tdms";

    let w = TdmsWriter::create(test_file)?;
    w.abort()?;

    // File should not exist
    assert!(!std::path::Path::new(test_file).exists());

    Ok(())
}
