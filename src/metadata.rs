

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPath {
    pub raw: String,
    pub components: Vec<String>,
}

impl ObjectPath {
    pub fn new(path: String) -> Self {
        // Path format: /'Group'/'Channel'
        // We need to parse this carefully.
        // nptdms uses ' to quote components.
        
        // Simple manual parser for now
        let mut components = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;
        
        for c in path.chars() {
            match c {
                '/' => {
                    if in_quote {
                        current.push(c);
                    } else if !current.is_empty() {
                         // End of previous component
                         // We expect components to be quoted like 'Name'
                         // But we want to store just Name
                         components.push(trim_quotes(&current));
                         current.clear();
                    }
                }
                '\'' => {
                    in_quote = !in_quote;
                    current.push(c);
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            components.push(trim_quotes(&current));
        }

        Self {
            raw: path,
            components,
        }
    }
    
    pub fn group_name(&self) -> Option<&str> {
        if self.components.len() >= 1 {
            Some(&self.components[0])
        } else {
            None
        }
    }

    pub fn channel_name(&self) -> Option<&str> {
        if self.components.len() >= 2 {
            Some(&self.components[1])
        } else {
            None
        }
    }
    
    pub fn is_root(&self) -> bool {
        self.raw == "/"
    }
}

fn trim_quotes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

use std::collections::HashMap;
use crate::datatypes::{DataType, TdmsData, PropertyValue};

#[derive(Debug, Clone)]
pub struct RawDataMeta {
    pub data_type: DataType,
    pub dimension: u32,
    pub number_of_values: u64,
    pub total_size_bytes: Option<u64>, // For variable length types
}

#[derive(Debug)]
pub struct ParsingMetadata {
    pub path: ObjectPath,
    pub raw_data_index: u32,
    pub properties: HashMap<String, PropertyValue>,
    pub raw_data_meta: Option<RawDataMeta>,
    pub data: Option<TdmsData>,
}

