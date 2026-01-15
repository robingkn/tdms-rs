//! TDMS file writing with properties example.
//!
//! This example demonstrates how to add properties at file, group, and channel levels.
//! Properties provide metadata about the data and are commonly used in TDMS files
//! for units, descriptions, calibration information, and more.

use std::fs;
use tdms_rs::{PropertyValue, TdmsFile, TdmsWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("📝 Creating TDMS file with properties...");

    // Create a new TDMS file writer
    let mut writer = TdmsWriter::create("examples/output/with_properties.tdms")?;

    // Add file-level properties (using new From<T> conversions)
    writer.add_property("Author", PropertyValue::String("TDMS Writer Example".into()))?;
    writer.add_property("Version", PropertyValue::I32(1))?;
    writer.add_property("Creation_Date", PropertyValue::String("2026-01-09".into()))?;
    writer.add_property(
        "Description",
        PropertyValue::String("Example file with comprehensive properties".into()),
    )?;

    // Add a measurements group
    let mut measurements = writer.add_group("Measurements")?;

    // Add group-level properties (using new From<T> conversions)
    measurements.add_property("Unit_System", PropertyValue::String("SI".into()))?;
    measurements.add_property("Sample_Rate", PropertyValue::Double(1000.0))?;
    measurements.add_property("Duration", PropertyValue::Double(0.01))?;
    measurements.add_property("Channels", PropertyValue::I32(3))?;

    // Voltage channel with properties
    let mut voltage_channel = measurements.add_channel::<f64>("Voltage")?;
    voltage_channel.write(&[1.1, 2.2, 3.3, 4.4, 5.5, 4.4, 3.3, 2.2, 1.1, 0.0])?;
    voltage_channel.add_property("wf_unit_string", PropertyValue::String("V".into()))?;
    voltage_channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
    voltage_channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    voltage_channel.add_property("Description", PropertyValue::String("AC voltage measurement".into()))?;
    voltage_channel.add_property("Range", PropertyValue::String("±10V".into()))?;
    voltage_channel.add_property("Calibrated", PropertyValue::Boolean(true))?;

    // Current channel with properties
    let mut current_channel = measurements.add_channel::<f32>("Current")?;
    current_channel.write(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0])?;
    current_channel.add_property("wf_unit_string", PropertyValue::String("A".into()))?;
    current_channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
    current_channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    current_channel.add_property("Description", PropertyValue::String("AC current measurement".into()))?;
    current_channel.add_property("Range", PropertyValue::String("±1A".into()))?;
    current_channel.add_property("Shunt_Resistance", PropertyValue::Double(0.1))?;

    // Temperature channel with properties
    let mut temp_channel = measurements.add_channel::<i32>("Temperature")?;
    temp_channel.write(&[2010, 2015, 2020, 2025, 2030, 2025, 2020, 2015, 2010, 2005])?;
    temp_channel.add_property("wf_unit_string", PropertyValue::String("°C".into()))?;
    temp_channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
    temp_channel.add_property("wf_start_time", PropertyValue::Double(0.0))?;
    temp_channel.add_property("Description", PropertyValue::String("Thermocouple temperature".into()))?;
    temp_channel.add_property("Sensor_Type", PropertyValue::String("K-Type Thermocouple".into()))?;
    temp_channel.add_property("Scale_Factor", PropertyValue::Double(0.01))?;
    temp_channel.add_property("Offset", PropertyValue::Double(0.0))?;

    // Add a status group
    let mut status = writer.add_group("Status")?;
    status.add_property(
        "Purpose",
        PropertyValue::String("System status and diagnostics".into()),
    )?;

    // String channels are intentionally not written by this example because the redesigned
    // writer API only supports fixed-size primitive types and does not implement string encoding.

    // Error flags channel
    let mut error_channel = status.add_channel::<bool>("Error_Flags")?;
    error_channel.write(&[false, false, false, false, false, true, false, false, false, false])?;
    error_channel.add_property("Description", PropertyValue::String("Error condition flags".into()))?;
    error_channel.add_property("Error_Code", PropertyValue::I32(0x0020))?;

    // Write the file
    writer.close()?;

    println!("✅ Successfully created 'examples/output/with_properties.tdms'");
    println!("   - File properties: 4");
    println!("   - 2 groups: 'Measurements', 'Status'");
    println!("   - 5 channels with comprehensive properties");

    // Verify by reading it back and displaying properties
    println!("\n🔍 Verifying properties by reading the file back...");
    let file = TdmsFile::open(std::path::Path::new("examples/output/with_properties.tdms"))?;

    // Display file-level properties
    println!("   File Properties:");
    let prop_count = file.properties().count();
    if prop_count == 0 {
        println!("     No file-level properties");
    } else {
        for (prop_name, prop_value) in file.properties() {
            println!("     {}: {:?}", prop_name, prop_value);
        }
    }

    for group in file.groups() {
        println!("   Group: {}", group.name());

        if group.properties().count() > 0 {
            println!("     Group Properties:");
            for (prop_name, prop_value) in group.properties() {
                println!("       {}: {:?}", prop_name, prop_value);
            }
        }

        for channel in group.channels() {
            println!("     Channel: {}", channel.name());
            println!("       Data: {} samples ({:?})", channel.len(), channel.dtype());

            if channel.properties().count() > 0 {
                println!("       Channel Properties:");
                for (prop_name, prop_value) in channel.properties() {
                    println!("         {}: {:?}", prop_name, prop_value);
                }
            }
        }
    }

    println!("\n✨ Properties verification successful!");
    println!("💡 File-level properties are fully supported in both reading and writing!");

    Ok(())
}
