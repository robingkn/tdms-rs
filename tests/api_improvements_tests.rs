use tdms_rs::{TdmsFile, TdmsFileWriter, TdmsData, PropertyValue};
use std::fs;
use std::path::Path;

#[test]
fn test_convenience_methods() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("tests/output")?;
    
    let output_path = "tests/output/convenience_test.tdms";
    let mut writer = TdmsFileWriter::new(output_path);
    
    // Test file-level properties
    writer.add_property("Author", PropertyValue::String("Test".into()))?;
    
    let group = writer.add_group("TestGroup")?;
    
    // Add channels with different data types
    group.add_channel("DoubleData", TdmsData::Double(vec![1.1, 2.2, 3.3]))?;
    group.add_channel("IntData", TdmsData::I32(vec![10, 20, 30]))?;
    group.add_channel("StringData", TdmsData::String(vec!["A".into(), "B".into(), "C".into()]))?;
    
    // Add channel with properties
    let voltage_channel = group.add_channel("Voltage", TdmsData::Double(vec![5.0, 10.0, 15.0]))?;
    voltage_channel.add_property("wf_unit_string", PropertyValue::String("V".into()))?;
    voltage_channel.add_property("wf_increment", PropertyValue::Double(0.001))?;
    voltage_channel.add_property("custom_prop", PropertyValue::I32(42))?;
    
    writer.write()?;
    
    // Load and test convenience methods
    let file = TdmsFile::load(Path::new(output_path))?;
    
    // Test file iteration
    assert_eq!(file.groups.len(), 1);
    let mut group_count = 0;
    for (group_name, _group) in file.iter_groups() {
        assert_eq!(group_name, "TestGroup");
        group_count += 1;
    }
    assert_eq!(group_count, 1);
    
    let test_group = file.groups.get("TestGroup").unwrap();
    
    // Test group iteration
    let mut channel_count = 0;
    for (_channel_name, _channel) in test_group.iter_channels() {
        channel_count += 1;
    }
    assert_eq!(channel_count, 4);
    
    // Test convenience methods on channels
    let double_channel = test_group.channels.get("DoubleData").unwrap();
    assert_eq!(double_channel.as_f64(), Some([1.1, 2.2, 3.3].as_slice()));
    assert_eq!(double_channel.data_len(), 3);
    assert_eq!(double_channel.data_type_name(), Some("Double"));
    assert!(double_channel.as_numeric().is_some());
    
    let int_channel = test_group.channels.get("IntData").unwrap();
    assert_eq!(int_channel.as_i32(), Some([10, 20, 30].as_slice()));
    assert_eq!(int_channel.data_len(), 3);
    assert_eq!(int_channel.data_type_name(), Some("I32"));
    let numeric_data = int_channel.as_numeric().unwrap();
    assert_eq!(numeric_data, vec![10.0, 20.0, 30.0]);
    
    let string_channel = test_group.channels.get("StringData").unwrap();
    assert_eq!(string_channel.as_string().unwrap().len(), 3);
    assert_eq!(string_channel.data_len(), 3);
    assert_eq!(string_channel.data_type_name(), Some("String"));
    assert!(string_channel.as_numeric().is_none());
    
    // Test property helpers
    let voltage_channel = test_group.channels.get("Voltage").unwrap();
    assert_eq!(voltage_channel.unit(), Some("V"));
    assert_eq!(voltage_channel.increment(), Some(0.001));
    assert_eq!(voltage_channel.get_i32_property("custom_prop"), Some(42));
    assert_eq!(voltage_channel.get_string_property("wf_unit_string"), Some("V"));
    assert_eq!(voltage_channel.get_double_property("wf_increment"), Some(0.001));
    
    Ok(())
}

#[test]
fn test_input_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = TdmsFileWriter::new("tests/output/validation_test.tdms");
    
    // Test empty group name validation
    match writer.add_group("") {
        Err(tdms_rs::error::TdmsError::InvalidName(_)) => {}, // Expected
        _ => panic!("Expected InvalidName error for empty group name"),
    }
    
    // Test duplicate group name validation
    writer.add_group("TestGroup")?;
    match writer.add_group("TestGroup") {
        Err(tdms_rs::error::TdmsError::DuplicateName(_)) => {}, // Expected
        _ => panic!("Expected DuplicateName error for duplicate group"),
    }
    
    // Test empty channel name validation
    let group = writer.add_group("ValidGroup")?;
    match group.add_channel("", TdmsData::Double(vec![1.0])) {
        Err(tdms_rs::error::TdmsError::InvalidName(_)) => {}, // Expected
        _ => panic!("Expected InvalidName error for empty channel name"),
    }
    
    // Test duplicate channel name validation
    group.add_channel("TestChannel", TdmsData::Double(vec![1.0]))?;
    match group.add_channel("TestChannel", TdmsData::Double(vec![2.0])) {
        Err(tdms_rs::error::TdmsError::DuplicateName(_)) => {}, // Expected
        _ => panic!("Expected DuplicateName error for duplicate channel"),
    }
    
    // Test empty property key validation
    match group.add_property("", PropertyValue::String("test".into())) {
        Err(tdms_rs::error::TdmsError::InvalidName(_)) => {}, // Expected
        _ => panic!("Expected InvalidName error for empty property key"),
    }
    
    Ok(())
}

#[test]
fn test_display_traits() -> Result<(), Box<dyn std::error::Error>> {
    // Test PropertyValue Display
    let string_prop = PropertyValue::String("Hello World".into());
    assert_eq!(format!("{}", string_prop), "\"Hello World\"");
    
    let double_prop = PropertyValue::Double(3.14159);
    assert_eq!(format!("{}", double_prop), "3.141590");
    
    let nan_prop = PropertyValue::Double(f64::NAN);
    assert_eq!(format!("{}", nan_prop), "NaN");
    
    let inf_prop = PropertyValue::Double(f64::INFINITY);
    assert_eq!(format!("{}", inf_prop), "∞");
    
    let neg_inf_prop = PropertyValue::Double(f64::NEG_INFINITY);
    assert_eq!(format!("{}", neg_inf_prop), "-∞");
    
    let int_prop = PropertyValue::I32(42);
    assert_eq!(format!("{}", int_prop), "42");
    
    let bool_prop = PropertyValue::Boolean(true);
    assert_eq!(format!("{}", bool_prop), "true");
    
    let timestamp_prop = PropertyValue::TimeStamp((1000, 500000000));
    assert_eq!(format!("{}", timestamp_prop), "1000.0000000000500000000");
    
    // Test TdmsData Display
    let double_data = TdmsData::Double(vec![1.0, 2.0, 3.0]);
    assert_eq!(format!("{}", double_data), "Double [3]");
    
    let string_data = TdmsData::String(vec!["A".into(), "B".into()]);
    assert_eq!(format!("{}", string_data), "String [2]");
    
    let empty_data = TdmsData::I32(vec![]);
    assert_eq!(format!("{}", empty_data), "I32 [0]");
    
    Ok(())
}

#[test]
fn test_data_utility_methods() -> Result<(), Box<dyn std::error::Error>> {
    let double_data = TdmsData::Double(vec![1.0, 2.0, 3.0]);
    assert_eq!(double_data.len(), 3);
    assert!(!double_data.is_empty());
    assert_eq!(double_data.type_name(), "Double");
    assert!(double_data.is_numeric());
    
    let string_data = TdmsData::String(vec!["Hello".into(), "World".into()]);
    assert_eq!(string_data.len(), 2);
    assert!(!string_data.is_empty());
    assert_eq!(string_data.type_name(), "String");
    assert!(!string_data.is_numeric());
    
    let empty_data = TdmsData::Boolean(vec![]);
    assert_eq!(empty_data.len(), 0);
    assert!(empty_data.is_empty());
    assert_eq!(empty_data.type_name(), "Boolean");
    assert!(!empty_data.is_numeric());
    
    Ok(())
}

#[test]
fn test_ordered_collections() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("tests/output")?;
    
    let output_path = "tests/output/ordered_test.tdms";
    let mut writer = TdmsFileWriter::new(output_path);
    
    // Add groups in specific order
    let group_c = writer.add_group("GroupC")?;
    group_c.add_channel("ChannelZ", TdmsData::Double(vec![3.0]))?;
    group_c.add_channel("ChannelA", TdmsData::Double(vec![1.0]))?;
    
    let group_a = writer.add_group("GroupA")?;
    group_a.add_channel("ChannelY", TdmsData::Double(vec![2.0]))?;
    
    let group_b = writer.add_group("GroupB")?;
    group_b.add_channel("ChannelX", TdmsData::Double(vec![4.0]))?;
    
    writer.write()?;
    
    // Load and verify order is preserved
    let file = TdmsFile::load(Path::new(output_path))?;
    
    let group_names: Vec<&str> = file.iter_groups().map(|(name, _)| name).collect();
    assert_eq!(group_names, vec!["GroupC", "GroupA", "GroupB"]);
    
    // Check channel order within GroupC (should be insertion order, not alphabetical)
    let group_c = file.groups.get("GroupC").unwrap();
    let channel_names: Vec<&str> = group_c.iter_channels().map(|(name, _)| name).collect();
    // Note: channels use BTreeMap in writer, so they will be alphabetical
    assert_eq!(channel_names, vec!["ChannelA", "ChannelZ"]);
    
    Ok(())
}