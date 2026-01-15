use std::sync::Arc;
use std::thread;
use tdms::TdmsFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = Arc::new(TdmsFile::open("big.tdms")?);

    let handles: Vec<_> = (0..4).map(|i| {
        let f = f.clone();
        thread::spawn(move || {
            let ch = f.group("G").unwrap().channel("C").unwrap();
            let slice = ch.read(i*1_000_000..(i+1)*1_000_000).unwrap();
            slice.len()
        })
    }).collect();

    for h in handles {
        println!("read {}", h.join().unwrap());
    }
    Ok(())
}
