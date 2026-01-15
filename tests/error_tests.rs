use std::fs;
use std::io::Write;
use std::path::Path;
use tdms_rs::{TdmsError, TdmsFile};

#[test]
fn test_invalid_signature() {
    // Create a file with invalid signature
    let test_path = "tests/output/invalid_signature.tdms";
    fs::create_dir_all("tests/output").unwrap();

    let mut file = fs::File::create(test_path).unwrap();
    file.write_all(b"XXXX").unwrap(); // Invalid signature instead of "TDSm"
    file.write_all(&[0u8; 24]).unwrap(); // Rest of header
    drop(file);

    let result = TdmsFile::open(Path::new(test_path));
    match result {
        Err(TdmsError::InvalidSignature) => {} // Expected
        Err(e) => panic!("Expected InvalidSignature, got: {:?}", e),
        Ok(_) => panic!("Expected error, but file loaded successfully"),
    }
}

#[test]
fn test_nonexistent_file() {
    let result = TdmsFile::open(Path::new("tests/output/nonexistent.tdms"));
    match result {
        Err(TdmsError::Io(_)) => {} // Expected I/O error for missing file
        Err(e) => panic!("Expected I/O error, got: {:?}", e),
        Ok(_) => panic!("Expected error, but file loaded successfully"),
    }
}

// Note: Additional error cases (truncated files, invalid UTF-8, unsupported data types)
// would require more sophisticated test file construction and may need improvements
// to the reader's error handling. The above tests cover the basic error paths
// that are most likely to occur in practice.
