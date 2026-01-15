//! Debug test to see what's being written

use tdms_rs::{TdmsWriter, TdmsFile};

#[test]
fn test_debug_write() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test_debug.tdms";
    
    // Write
    {
        let mut w = TdmsWriter::create(test_file)?;
        let mut g = w.add_group("TestGroup")?;
        let mut ch = g.add_channel::<f64>("TestChannel")?;
        ch.write(&[1.0, 2.0, 3.0])?;
        w.close()?;
    }
    
    // Read and debug
    {
        let f = TdmsFile::open(test_file)?;
        
        println!("Groups found:");
        for g in f.groups() {
            println!("  - '{}'", g.name());
            for ch in g.channels() {
                println!("    - '{}'", ch.name());
            }
        }
    }
    
    // Clean up
    std::fs::remove_file(test_file).ok();
    
    Ok(())
}
