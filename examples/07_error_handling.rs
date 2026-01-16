//! Demonstrates error handling patterns for TDMS write operations.
//! Shows how to handle errors gracefully without using abort.

use std::path::Path;
use tdms_rs::{PropertyValue, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("error_handling_example.tdms");

    // Example 1: Basic error handling with early return
    println!("Example 1: Basic error handling");
    match basic_write_with_validation(path) {
        Ok(_) => println!("✓ File written successfully"),
        Err(e) => println!("✗ Write failed: {}", e),
    }

    // Clean up between examples
    let _ = std::fs::remove_file(path);

    // Example 2: Conditional writing with validation
    println!("\nExample 2: Conditional writing with validation");
    match conditional_write_with_validation(path) {
        Ok(_) => println!("✓ File written successfully"),
        Err(e) => println!("✗ Write failed: {}", e),
    }

    // Clean up
    let _ = std::fs::remove_file(path);
    Ok(())
}

/// Example: Basic write operation with validation and error handling
fn basic_write_with_validation(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsWriter::create(path)?;

    // Add some metadata
    writer.add_property("author", PropertyValue::String("Example".into()))?;

    let mut group = writer.add_group("Measurements")?;
    let mut channel = group.add_channel::<f64>("Voltage")?;

    // Simulate some data that might fail validation
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    // Validate data before writing
    if data.is_empty() {
        return Err("Data cannot be empty".into());
    }

    if data.iter().any(|&v: &f64| v.is_nan() || v.is_infinite()) {
        return Err("Data contains invalid values".into());
    }

    channel.write(&data)?;

    // File is automatically flushed and closed when writer goes out of scope
    println!("Wrote {} data points", data.len());

    Ok(())
}

/// Example: Conditional writing with complex validation logic
fn conditional_write_with_validation(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsWriter::create(path)?;

    let mut group = writer.add_group("Sensors")?;

    // Simulate sensor data that might be invalid
    let temperatures = vec![20.0, 21.5, 22.0, 23.5, 24.0];
    let pressures = vec![101.3, 101.2, 101.1, 101.0, 100.9];

    // Validate that both channels have the same length
    if temperatures.len() != pressures.len() {
        return Err("Temperature and pressure channels must have same length".into());
    }

    // Validate temperature range (reasonable sensor values)
    if temperatures.iter().any(|&t| t < -50.0 || t > 150.0) {
        return Err("Temperature values out of reasonable range".into());
    }

    // Validate pressure range (reasonable atmospheric pressure in kPa)
    if pressures.iter().any(|&p| p < 50.0 || p > 150.0) {
        return Err("Pressure values out of reasonable range".into());
    }

    // If all validations pass, create channels and write the data
    {
        let mut temp_channel = group.add_channel::<f64>("Temperature")?;
        temp_channel.write(&temperatures)?;
    }

    {
        let mut pressure_channel = group.add_channel::<f64>("Pressure")?;
        pressure_channel.write(&pressures)?;
    }

    // File is automatically flushed and closed when writer goes out of scope
    println!("Wrote {} sensor readings", temperatures.len());

    Ok(())
}
