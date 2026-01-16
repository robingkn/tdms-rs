use crate::model::datatypes::{DataType, PropertyValue};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPath {
    pub raw: String,
    pub components: Vec<String>,
}

impl ObjectPath {
    pub fn new(path: String) -> Self {
        let mut components = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;

        for c in path.chars() {
            match c {
                '/' => {
                    if in_quote {
                        current.push(c);
                    } else if !current.is_empty() {
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
        self.components.first().map(|s| s.as_str())
    }

    pub fn channel_name(&self) -> Option<&str> {
        self.components.get(1).map(|s| s.as_str())
    }

    pub fn is_root(&self) -> bool {
        self.raw == "/"
    }
}

fn trim_quotes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct RawDataMeta {
    pub data_type: DataType,
    pub number_of_values: u64,
    pub total_size_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DataLocation {
    pub offset: u64,
    pub number_of_values: u64,
    pub data_type: DataType,
    pub total_size_bytes: Option<u64>,
}

#[derive(Debug)]
pub struct ParsingMetadata {
    pub path: ObjectPath,
    pub raw_data_index: u32,
    pub properties: HashMap<String, PropertyValue>,
    pub raw_data_meta: Option<RawDataMeta>,
    pub data_location: Option<DataLocation>,
}
