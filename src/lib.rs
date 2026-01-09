
pub mod reader;
pub mod segment;
pub mod metadata;
pub mod datatypes;
pub mod channel;
pub mod error;
pub mod utils;
pub mod writer;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use crate::error::Result;
use crate::reader::TdmsReader;

use std::collections::HashMap;

/// A TDMS file containing groups and channels with their associated data and properties.
/// 
/// TDMS (Technical Data Management Streaming) is a file format developed by National Instruments
/// for storing measurement data. A TDMS file has a hierarchical structure:
/// - File (root level)
/// - Groups (containers for related channels)  
/// - Channels (individual data streams with properties and data)
/// 
/// # Examples
/// 
/// ```no_run
/// use tdms_rs::TdmsFile;
/// use std::path::Path;
/// 
/// // Load a TDMS file
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
/// 
/// // Iterate through groups and channels
/// for (group_name, group) in &file.groups {
///     println!("Group: {}", group_name);
///     for (channel_name, channel) in &group.channels {
///         if let Some(data) = &channel.data {
///             let sample_count = match data {
///                 tdms_rs::TdmsData::Double(v) => v.len(),
///                 tdms_rs::TdmsData::I32(v) => v.len(),
///                 _ => 0, // Handle other types as needed
///             };
///             println!("  Channel {}: {} samples", channel_name, sample_count);
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsFile {
    /// Groups contained in this TDMS file, indexed by group name.
    /// 
    /// Group names are the path components from the TDMS file structure.
    /// For example, a path like `/'Sensors'/'Temperature'` creates a group named "Sensors"
    /// containing a channel named "Temperature".
    pub groups: HashMap<String, TdmsGroup>,
    // Placeholder structure
    path: String,
}

pub use crate::datatypes::{PropertyValue, TdmsData};
pub use crate::writer::{TdmsFileWriter, TdmsGroupWriter, TdmsChannelWriter};

/// A group within a TDMS file containing related channels and group-level properties.
/// 
/// Groups serve as containers for organizing related measurement channels.
/// They can have their own properties (metadata) and contain multiple channels.
/// 
/// # Examples
/// 
/// ```no_run
/// use tdms_rs::TdmsFile;
/// use std::path::Path;
/// 
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
/// 
/// if let Some(group) = file.groups.get("Sensors") {
///     println!("Group has {} properties", group.properties.len());
///     println!("Group has {} channels", group.channels.len());
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsGroup {
    /// Properties (metadata) associated with this group.
    /// 
    /// Properties are key-value pairs that provide metadata about the group.
    /// Common properties might include creation time, description, or custom metadata.
    pub properties: HashMap<String, PropertyValue>,
    
    /// Channels contained in this group, indexed by channel name.
    /// 
    /// Each channel represents a single data stream (e.g., a sensor reading over time).
    pub channels: HashMap<String, TdmsChannel>,
}

/// A channel within a TDMS group containing data and channel-level properties.
/// 
/// Channels represent individual data streams, such as sensor readings, timestamps,
/// or calculated values. Each channel has associated metadata (properties) and
/// may contain actual measurement data.
/// 
/// # Examples
/// 
/// ```no_run
/// use tdms_rs::{TdmsFile, TdmsData};
/// use std::path::Path;
/// 
/// let file = TdmsFile::load(Path::new("data.tdms"))?;
/// 
/// if let Some(group) = file.groups.get("Sensors") {
///     if let Some(channel) = group.channels.get("Temperature") {
///         // Access channel data
///         match &channel.data {
///             Some(TdmsData::Double(values)) => {
///                 println!("Temperature readings: {} samples", values.len());
///                 if let Some(first) = values.first() {
///                     println!("First reading: {:.2}°C", first);
///                 }
///             },
///             Some(other) => println!("Unexpected data type: {:?}", other),
///             None => println!("No data in channel"),
///         }
///         
///         // Access channel properties
///         if let Some(unit) = channel.properties.get("wf_unit_string") {
///             println!("Unit: {:?}", unit);
///         }
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TdmsChannel {
    /// Properties (metadata) associated with this channel.
    /// 
    /// Channel properties contain metadata about the data stream, such as:
    /// - `wf_unit_string`: Physical unit of measurement (e.g., "°C", "V", "Hz")
    /// - `wf_increment`: Time increment between samples for waveform data
    /// - `wf_start_time`: Start time for waveform data
    /// - `wf_samples`: Number of samples in the waveform
    /// - Custom properties defined by the application
    pub properties: HashMap<String, PropertyValue>,
    
    /// The actual measurement data for this channel.
    /// 
    /// Data is `None` if the channel contains only metadata (no actual measurements).
    /// When present, data is stored in a type-safe enum that preserves the original
    /// TDMS data type (integers, floats, strings, timestamps, etc.).
    pub data: Option<TdmsData>,
}

impl TdmsFile {
    /// Load a TDMS file from the specified path.
    /// 
    /// This method reads and parses the entire TDMS file, extracting all groups,
    /// channels, properties, and data. The file is read sequentially, handling
    /// multiple segments if present.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the TDMS file to load
    /// 
    /// # Returns
    /// 
    /// Returns a `TdmsFile` containing all parsed data, or an error if the file
    /// cannot be read or parsed.
    /// 
    /// # Errors
    /// 
    /// This method can return errors for various reasons:
    /// - File not found or permission denied
    /// - Invalid TDMS file format or corrupted data
    /// - Unsupported TDMS features or data types
    /// - I/O errors during reading
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use tdms_rs::TdmsFile;
    /// use std::path::Path;
    /// 
    /// // Load a TDMS file
    /// let file = TdmsFile::load(Path::new("measurements.tdms"))?;
    /// println!("Loaded {} groups", file.groups.len());
    /// 
    /// // Handle errors gracefully
    /// match TdmsFile::load(Path::new("missing.tdms")) {
    ///     Ok(file) => println!("File loaded successfully"),
    ///     Err(e) => eprintln!("Failed to load file: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = TdmsReader::new(BufReader::new(file));
        
        let mut groups = HashMap::new();
        
        loop {
            let segment = match reader.read_segment() {
                Ok(s) => s,
                Err(crate::error::TdmsError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            for obj in segment.objects {
                // If path has 0 components -> Root
                // If path has 1 component -> Group
                // If path has 2 components -> Channel
                
                if let Some(g_name) = obj.path.group_name() {
                    // Ensure group exists
                    let group = groups.entry(g_name.to_string()).or_insert(TdmsGroup {
                        properties: HashMap::new(),
                        channels: HashMap::new(),
                    });
                    
                    if let Some(c_name) = obj.path.channel_name() {
                        // Add channel
                        let channel = group.channels.entry(c_name.to_string()).or_insert(TdmsChannel {
                             properties: HashMap::new(),
                             data: None, 
                        });
                        channel.properties.extend(obj.properties);
                        if let Some(new_data) = obj.data {
                            if let Some(existing_data) = &mut channel.data {
                                existing_data.extend(new_data)?;
                            } else {
                                channel.data = Some(new_data);
                            }
                        }
                    } else {
                        // Group properties
                        group.properties.extend(obj.properties);
                    }
                } else if obj.path.is_root() {
                    // File properties (store where?)
                }
            }
        }
        
        // Return populated groups
        Ok(Self {
            groups,
            path: path.to_string_lossy().to_string(),
        })
    }
}
