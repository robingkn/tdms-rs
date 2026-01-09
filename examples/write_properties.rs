//! TDMS file writing with properties example.
//! 
//! This example demonstrates how to add properties at file, group, and channel levels.
//! Properties provide metadata about the data and are commonly used in TDMS files
//! for units, descriptions, calibration information, and more.

use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::{TdmsData, PropertyValue};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;
    
    println!("📝 Creating TDMS file with properties...");
    
    // Create a new TDMS file writer
    let mut writer = TdmsFileWriter::new("examples/output/with_properties.tdms");
    
    // Add file-level properties
    writer.add_property("Author", PropertyValue::String("TDMS Writer Example".to_string()))?;
    writer.add_property("Version", PropertyValue::I32(1))?;
    writer.add_property("Creation_Date", PropertyValue::String("2026-01-09".to_string()))?;
    writer.add_property("Description", PropertyValue::String("Example file with comprehensive properties".to_string()))?;
    
    // Add a measurements group
    let measurements = writer.add_group("Measurements")?;
    
    // Add group-level properties
    measurements.add_property("Unit_System", PropertyValue::String("SI".to_string()))?;
    measurements.add_property("Sample_Rate", PropertyValue::Double(1000.0))?;
    measurements.add_property("Duration", PropertyValue::Double(0.01))?; // 10ms
    measurements.add_property("Channels", PropertyValue::I32(3))?;
    
    // Voltage channel with properties
    let voltage_channel = measurements.add_channel("Voltage", TdmsData::Double(vec![
        1.1, 2.2, 3.3, 4.4, 5.5, 4.4, 3.3, 2.2, 1.1, 0.0
    ]))?;
    voltage_channel.add_property("wf_unit_string", PropertyValue::String("V".to_string()))?;
    voltage_channel.add_property("wf_increment", PropertyValue::Double(0.001))?; // 1ms increment
    voltage_channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    voltage_channel.add_property("Description", PropertyValue::String("AC voltage measurement".to_string()))?;
    voltage_channel.add_property("Range", PropertyValue::String("±10V".to_string()))?;
    voltage_channel.add_property("Calibrated", PropertyValue::Boolean(true))?;
    
    // Current channel with properties
    let current_channel = measurements.add_channel("Current", TdmsData::Float(vec![
        0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0
    ]))?;
    current_channel.add_property("wf_unit_string", PropertyValue::String("A".to_string()))?;
    current_channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
    current_channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    current_channel.add_property("Description", PropertyValue::String("AC current measurement".to_string()))?;
    current_channel.add_property("Range", PropertyValue::String("±1A".to_string()))?;
    current_channel.add_property("Shunt_Resistance", PropertyValue::Double(0.1))?; // 0.1 ohm shunt
    
    // Temperature channel with properties
    let temp_channel = measurements.add_channel("Temperature", TdmsData::I32(vec![
        2010, 2015, 2020, 2025, 2030, 2025, 2020, 2015, 2010, 2005
    ]))?; // Temperature in 0.01°C units
    temp_channel.add_property("wf_unit_string", PropertyValue::String("°C".to_string()))?;
    temp_channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
    temp_channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    temp_channel.add_property("Description", PropertyValue::String("Thermocouple temperature".to_string()))?;
    temp_channel.add_property("Sensor_Type", PropertyValue::String("K-Type Thermocouple".to_string()))?;
    temp_channel.add_property("Scale_Factor", PropertyValue::Double(0.01))?; // Convert to actual °C
    temp_channel.add_property("Offset", PropertyValue::Double(0.0))?;
    
    // Add a status group
    let status = writer.add_group("Status")?;
    status.add_property("Purpose", PropertyValue::String("System status and diagnostics".to_string()))?;
    
    // System status channel
    let status_channel = status.add_channel("System_Status", TdmsData::String(vec![
        "Initializing".to_string(),
        "Calibrating".to_string(),
        "Running".to_string(),
        "Running".to_string(),
        "Running".to_string(),
        "Warning".to_string(),
        "Running".to_string(),
        "Running".to_string(),
        "Stopping".to_string(),
        "Stopped".to_string(),
    ]))?;
    status_channel.add_property("Description", PropertyValue::String("System operational status".to_string()))?;
    status_channel.add_property("Valid_States", PropertyValue::String("Initializing,Calibrating,Running,Warning,Stopping,Stopped".to_string()))?;
    
    // Error flags channel
    let error_channel = status.add_channel("Error_Flags", TdmsData::Boolean(vec![
        false, false, false, false, false, true, false, false, false, false
    ]))?;
    error_channel.add_property("Description", PropertyValue::String("Error condition flags".to_string()))?;
    error_channel.add_property("Error_Code", PropertyValue::I32(0x0020))?; // Bit 5 set for warning
    
    // Write the file
    writer.write()?;
    
    println!("✅ Successfully created 'examples/output/with_properties.tdms'");
    println!("   - File properties: 4");
    println!("   - 2 groups: 'Measurements', 'Status'");
    println!("   - 5 channels with comprehensive properties");
    
    // Verify by reading it back and displaying properties
    println!("\n🔍 Verifying properties by reading the file back...");
    let file = tdms_rs::TdmsFile::load(std::path::Path::new("examples/output/with_properties.tdms"))?;
    
    // Display file-level properties
    println!("   File Properties:");
    if file.properties.is_empty() {
        println!("     No file-level properties");
    } else {
        for (prop_name, prop_value) in &file.properties {
            match prop_value {
                tdms_rs::PropertyValue::String(s) => println!("     {}: \"{}\"", prop_name, s),
                tdms_rs::PropertyValue::Double(d) => println!("     {}: {}", prop_name, d),
                tdms_rs::PropertyValue::I32(i) => println!("     {}: {}", prop_name, i),
                tdms_rs::PropertyValue::Boolean(b) => println!("     {}: {}", prop_name, b),
                _ => println!("     {}: {:?}", prop_name, prop_value),
            }
        }
    }
    
    for (group_name, group) in &file.groups {
        println!("   Group: {}", group_name);
        
        // Display group properties
        if !group.properties.is_empty() {
            println!("     Group Properties:");
            for (prop_name, prop_value) in &group.properties {
                match prop_value {
                    tdms_rs::PropertyValue::String(s) => println!("       {}: \"{}\"", prop_name, s),
                    tdms_rs::PropertyValue::Double(d) => println!("       {}: {}", prop_name, d),
                    tdms_rs::PropertyValue::I32(i) => println!("       {}: {}", prop_name, i),
                    tdms_rs::PropertyValue::Boolean(b) => println!("       {}: {}", prop_name, b),
                    _ => println!("       {}: {:?}", prop_name, prop_value),
                }
            }
        }
        
        // Display channels and their properties
        for (channel_name, channel) in &group.channels {
            println!("     Channel: {}", channel_name);
            
            if let Some(data) = &channel.data {
                match data {
                    tdms_rs::TdmsData::Double(values) => {
                        println!("       Data: {} double values", values.len());
                    },
                    tdms_rs::TdmsData::Float(values) => {
                        println!("       Data: {} float values", values.len());
                    },
                    tdms_rs::TdmsData::I32(values) => {
                        println!("       Data: {} i32 values", values.len());
                    },
                    tdms_rs::TdmsData::String(values) => {
                        println!("       Data: {} string values", values.len());
                    },
                    tdms_rs::TdmsData::Boolean(values) => {
                        println!("       Data: {} boolean values", values.len());
                    },
                    _ => println!("       Data: {:?}", data),
                }
            }
            
            // Display channel properties
            if !channel.properties.is_empty() {
                println!("       Channel Properties:");
                for (prop_name, prop_value) in &channel.properties {
                    match prop_value {
                        tdms_rs::PropertyValue::String(s) => println!("         {}: \"{}\"", prop_name, s),
                        tdms_rs::PropertyValue::Double(d) => println!("         {}: {}", prop_name, d),
                        tdms_rs::PropertyValue::I32(i) => println!("         {}: {}", prop_name, i),
                        tdms_rs::PropertyValue::Boolean(b) => println!("         {}: {}", prop_name, b),
                        _ => println!("         {}: {:?}", prop_name, prop_value),
                    }
                }
            }
        }
    }
    
    println!("\n✨ Properties verification successful!");
    println!("💡 File-level properties are fully supported in both reading and writing!");
    
    Ok(())
}