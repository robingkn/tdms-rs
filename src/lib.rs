
pub mod reader;
pub mod segment;
pub mod metadata;
pub mod datatypes;
pub mod channel;
pub mod error;
pub mod utils;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use crate::error::Result;
use crate::reader::TdmsReader;

use std::collections::HashMap;

pub struct TdmsFile {
    pub groups: HashMap<String, TdmsGroup>,
    // Placeholder structure
    path: String,
}

pub use crate::datatypes::{PropertyValue, TdmsData};

pub struct TdmsGroup {
    pub properties: HashMap<String, PropertyValue>,
    pub channels: HashMap<String, TdmsChannel>,
}

pub struct TdmsChannel {
    pub properties: HashMap<String, PropertyValue>,
    pub data: Option<TdmsData>,
}

impl TdmsFile {
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
