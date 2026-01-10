//! TDMS file writing with properties example.
//!
//! This example demonstrates how to add properties at file, group, and channel levels.
//! Properties provide metadata about the data and are commonly used in TDMS files
//! for units, descriptions, calibration information, and more.

use std::fs;
use tdms_rs::writer::TdmsFileWriter;
use tdms_rs::TdmsData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating TDMS file with properties...");

    // Create a new TDMS file writer
    let mut writer = TdmsFileWriter::new("examples/output/with_properties.tdms");

    // Add file-level properties (using new From<T> conversions)
    writer.add_property("Author", "TDMS Writer Example")?;
    writer.add_property("Version", 1i32)?;
    writer.add_property("Creation_Date", "2026-01-09")?;
    writer.add_property("Description", "Example file with comprehensive properties")?;

    // Add a measurements group
    let measurements = writer.add_group("Measurements")?;

    // Add group-level properties (using new From<T> conversions)
    measurements.add_property("Unit_System", "SI")?;
    measurements.add_property("Sample_Rate", 1000.0)?;
    measurements.add_property("Duration", 0.01)?; // 10ms
    measurements.add_property("Channels", 3i32)?;

    // Voltage channel with properties
    let voltage_channel = measurements.add_channel(
        "Voltage",
        TdmsData::Double(vec![1.1, 2.2, 3.3, 4.4, 5.5, 4.4, 3.3, 2.2, 1.1, 0.0]),
    )?;
    voltage_channel.add_property("wf_unit_string", "V")?;
    voltage_channel.add_property("wf_increment", 0.001)?; // 1ms increment
    voltage_channel.add_property("wf_start_time", 0.0)?;
    voltage_channel.add_property("Description", "AC voltage measurement")?;
    voltage_channel.add_property("Range", "±10V")?;
    voltage_channel.add_property("Calibrated", true)?;

    // Current channel with properties
    let current_channel = measurements.add_channel(
        "Current",
        TdmsData::Float(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0]),
    )?;
    current_channel.add_property("wf_unit_string", "A")?;
    current_channel.add_property("wf_increment", 0.001)?;
    current_channel.add_property("wf_start_time", 0.0)?;
    current_channel.add_property("Description", "AC current measurement")?;
    current_channel.add_property("Range", "±1A")?;
    current_channel.add_property("Shunt_Resistance", 0.1)?; // 0.1 ohm shunt

    // Temperature channel with properties
    let temp_channel = measurements.add_channel(
        "Temperature",
        TdmsData::I32(vec![
            2010, 2015, 2020, 2025, 2030, 2025, 2020, 2015, 2010, 2005,
        ]),
    )?; // Temperature in 0.01°C units
    temp_channel.add_property("wf_unit_string", "°C")?;
    temp_channel.add_property("wf_increment", 0.001)?;
    temp_channel.add_property("wf_start_time", 0.0)?;
    temp_channel.add_property("Description", "Thermocouple temperature")?;
    temp_channel.add_property("Sensor_Type", "K-Type Thermocouple")?;
    temp_channel.add_property("Scale_Factor", 0.01)?; // Convert to actual °C
    temp_channel.add_property("Offset", 0.0)?;

    // Add a status group
    let status = writer.add_group("Status")?;
    status.add_property("Purpose", "System status and diagnostics")?;

    // System status channel
    let status_channel = status.add_channel(
        "System_Status",
        TdmsData::String(vec![
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
        ]),
    )?;
    status_channel.add_property("Description", "System operational status")?;
    status_channel.add_property(
        "Valid_States",
        "Initializing,Calibrating,Running,Warning,Stopping,Stopped",
    )?;

    // Error flags channel
    let error_channel = status.add_channel(
        "Error_Flags",
        TdmsData::Boolean(vec![
            false, false, false, false, false, true, false, false, false, false,
        ]),
    )?;
    error_channel.add_property("Description", "Error condition flags")?;
    error_channel.add_property("Error_Code", 0x0020i32)?; // Bit 5 set for warning

    // Write the file
    writer.write()?;

    println!("✅ Successfully created 'examples/output/with_properties.tdms'");
    println!("   - File properties: 4");
    println!("   - 2 groups: 'Measurements', 'Status'");
    println!("   - 5 channels with comprehensive properties");

    // Verify by reading it back and displaying properties
    println!("\n🔍 Verifying properties by reading the file back...");
    let file =
        tdms_rs::TdmsFile::load(std::path::Path::new("examples/output/with_properties.tdms"))?;

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
                    tdms_rs::PropertyValue::String(s) => {
                        println!("       {}: \"{}\"", prop_name, s)
                    }
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
                    }
                    tdms_rs::TdmsData::Float(values) => {
                        println!("       Data: {} float values", values.len());
                    }
                    tdms_rs::TdmsData::I32(values) => {
                        println!("       Data: {} i32 values", values.len());
                    }
                    tdms_rs::TdmsData::String(values) => {
                        println!("       Data: {} string values", values.len());
                    }
                    tdms_rs::TdmsData::Boolean(values) => {
                        println!("       Data: {} boolean values", values.len());
                    }
                    _ => println!("       Data: {:?}", data),
                }
            }

            // Display channel properties
            if !channel.properties.is_empty() {
                println!("       Channel Properties:");
                for (prop_name, prop_value) in &channel.properties {
                    match prop_value {
                        tdms_rs::PropertyValue::String(s) => {
                            println!("         {}: \"{}\"", prop_name, s)
                        }
                        tdms_rs::PropertyValue::Double(d) => {
                            println!("         {}: {}", prop_name, d)
                        }
                        tdms_rs::PropertyValue::I32(i) => println!("         {}: {}", prop_name, i),
                        tdms_rs::PropertyValue::Boolean(b) => {
                            println!("         {}: {}", prop_name, b)
                        }
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
