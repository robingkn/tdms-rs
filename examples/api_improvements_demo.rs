//! API Improvements Demo
//! 
//! This example demonstrates the new API improvements in TDMS-RS:
//! - Input validation with proper error handling
//! - Convenience methods for data access
//! - Display trait implementations
//! - Ordered collections
//! - Property helper methods

use tdms_rs::{TdmsFile, TdmsFileWriter, TdmsData, PropertyValue};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;
    
    println!("🚀 TDMS-RS API Improvements Demo");
    println!("================================");
    
    // Demonstrate input validation
    println!("\n📋 Input Validation:");
    demonstrate_validation()?;
    
    // Create a sample file with various data types
    println!("\n📝 Creating sample file with improved API...");
    create_sample_file()?;
    
    // Demonstrate convenience methods
    println!("\n🔧 Convenience Methods:");
    demonstrate_convenience_methods()?;
    
    // Demonstrate Display traits
    println!("\n🖨️  Display Traits:");
    demonstrate_display_traits()?;
    
    println!("\n✅ Demo completed successfully!");
    Ok(())
}

fn demonstrate_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsFileWriter::new("examples/output/validation_demo.tdms");
    
    // Try to add empty group name (should fail)
    match writer.add_group("") {
        Err(e) => println!("   ❌ Empty group name rejected: {}", e),
        Ok(_) => println!("   ⚠️  Empty group name was accepted (unexpected)"),
    }
    
    // Add valid group
    let group = writer.add_group("ValidGroup")?;
    println!("   ✅ Valid group 'ValidGroup' added");
    
    // Try to add duplicate group (should fail)
    match writer.add_group("ValidGroup") {
        Err(e) => println!("   ❌ Duplicate group rejected: {}", e),
        Ok(_) => println!("   ⚠️  Duplicate group was accepted (unexpected)"),
    }
    
    // Get the group again for channel operations
    let group = writer.add_group("AnotherGroup")?;
    
    // Try empty channel name (should fail)
    match group.add_channel("", TdmsData::Double(vec![1.0])) {
        Err(e) => println!("   ❌ Empty channel name rejected: {}", e),
        Ok(_) => println!("   ⚠️  Empty channel name was accepted (unexpected)"),
    }
    
    // Add valid channel
    group.add_channel("ValidChannel", TdmsData::Double(vec![1.0, 2.0, 3.0]))?;
    println!("   ✅ Valid channel 'ValidChannel' added");
    
    // Try empty property key (should fail)
    match group.add_property("", PropertyValue::String("test".into())) {
        Err(e) => println!("   ❌ Empty property key rejected: {}", e),
        Ok(_) => println!("   ⚠️  Empty property key was accepted (unexpected)"),
    }
    
    Ok(())
}

fn create_sample_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsFileWriter::new("examples/output/api_demo.tdms");
    
    // Add file-level properties
    writer.add_property("Author", PropertyValue::String("API Demo".into()))?;
    writer.add_property("Version", PropertyValue::I32(2))?;
    writer.add_property("Created", PropertyValue::TimeStamp((1000, 0)))?;
    
    // Add sensors group
    let sensors = writer.add_group("Sensors")?;
    sensors.add_property("Location", PropertyValue::String("Lab A".into()))?;
    sensors.add_property("Sample_Rate", PropertyValue::Double(1000.0))?;
    
    // Add voltage channel with properties
    let voltage = sensors.add_channel("Voltage", TdmsData::Double(vec![
        1.1, 2.2, 3.3, 4.4, 5.5, 4.4, 3.3, 2.2, 1.1, 0.0
    ]))?;
    voltage.add_property("wf_unit_string", PropertyValue::String("V".into()))?;
    voltage.add_property("wf_increment", PropertyValue::Double(0.001))?;
    voltage.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    
    // Add temperature channel
    let temperature = sensors.add_channel("Temperature", TdmsData::I32(vec![
        200, 205, 210, 215, 220, 215, 210, 205, 200, 195
    ]))?;
    temperature.add_property("wf_unit_string", PropertyValue::String("°C×10".into()))?;
    temperature.add_property("Scale_Factor", PropertyValue::Double(0.1))?;
    
    // Add status channel
    sensors.add_channel("Status", TdmsData::String(vec![
        "OK".into(), "OK".into(), "WARNING".into(), "OK".into(), "OK".into()
    ]))?;
    
    // Add digital group
    let digital = writer.add_group("Digital")?;
    digital.add_channel("Flags", TdmsData::Boolean(vec![
        true, false, true, true, false, true, false, true
    ]))?;
    
    writer.write()?;
    println!("   ✅ Sample file created: examples/output/api_demo.tdms");
    
    Ok(())
}

fn demonstrate_convenience_methods() -> Result<(), Box<dyn std::error::Error>> {
    let file = TdmsFile::load(std::path::Path::new("examples/output/api_demo.tdms"))?;
    
    println!("   📊 File contains {} groups", file.groups.len());
    
    // Iterate through groups using new iterator
    for (group_name, group) in file.iter_groups() {
        println!("   📁 Group '{}' has {} channels", group_name, group.channels.len());
        
        // Iterate through channels using new iterator
        for (channel_name, channel) in group.iter_channels() {
            println!("      📈 Channel '{}': {} ({})", 
                channel_name, 
                channel.data_type_name().unwrap_or("No data"),
                channel.data_len()
            );
            
            // Demonstrate typed accessors
            if let Some(voltage_data) = channel.as_f64() {
                println!("         🔢 Double data: {} samples, first = {:.2}", 
                    voltage_data.len(), voltage_data[0]);
            }
            
            if let Some(temp_data) = channel.as_i32() {
                println!("         🔢 Integer data: {} samples, first = {}", 
                    temp_data.len(), temp_data[0]);
            }
            
            if let Some(string_data) = channel.as_string() {
                println!("         📝 String data: {} samples, first = '{}'", 
                    string_data.len(), string_data[0]);
            }
            
            // Demonstrate numeric conversion
            if let Some(numeric_data) = channel.as_numeric() {
                let avg = numeric_data.iter().sum::<f64>() / numeric_data.len() as f64;
                println!("         📊 Numeric average: {:.2}", avg);
            }
            
            // Demonstrate property helpers
            if let Some(unit) = channel.unit() {
                println!("         📏 Unit: {}", unit);
            }
            
            if let Some(increment) = channel.increment() {
                println!("         ⏱️  Increment: {}", increment);
            }
            
            if let Some(start_time) = channel.start_time() {
                println!("         🕐 Start time: {}", start_time);
            }
        }
    }
    
    Ok(())
}

fn demonstrate_display_traits() -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample property values
    let properties = vec![
        ("String", PropertyValue::String("Hello World".into())),
        ("Integer", PropertyValue::I32(42)),
        ("Double", PropertyValue::Double(3.14159)),
        ("Boolean", PropertyValue::Boolean(true)),
        ("NaN", PropertyValue::Double(f64::NAN)),
        ("Infinity", PropertyValue::Double(f64::INFINITY)),
        ("Timestamp", PropertyValue::TimeStamp((1000, 500000000))),
    ];
    
    println!("   🏷️  Property Values:");
    for (name, prop) in properties {
        println!("      {}: {}", name, prop);
    }
    
    // Create some sample data
    let data_samples = vec![
        TdmsData::Double(vec![1.0, 2.0, 3.0]),
        TdmsData::I32(vec![10, 20, 30, 40]),
        TdmsData::String(vec!["A".into(), "B".into()]),
        TdmsData::Boolean(vec![true, false, true]),
        TdmsData::U8(vec![]),  // Empty data
    ];
    
    println!("   📊 Data Types:");
    for data in data_samples {
        println!("      {}", data);
        println!("         Type: {}, Length: {}, Numeric: {}", 
            data.type_name(), data.len(), data.is_numeric());
    }
    
    Ok(())
}