
use std::env;
use std::path::PathBuf;
use tdms_rs::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tdms-to-json <file.tdms>");
        std::process::exit(1);
    }
    
    let path = PathBuf::from(&args[1]);
    println!("Loading {:?}", path);
    match TdmsFile::load(&path) {
        Ok(_) => println!("Loaded successfully (Preamble valid)"),
        Err(e) => println!("Failed to load: {}", e),
    }

    Ok(())
}
