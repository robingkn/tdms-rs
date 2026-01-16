use crate::model::datatypes::{DataType, PropertyValue};
use indexmap::IndexMap;

pub struct TdmsChannelData {
    pub name: String,
    pub dtype: DataType,
    pub len: usize,
    pub data_locations: Vec<DataLocation>,
    pub properties: IndexMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Copy)]
pub struct DataLocation {
    pub offset: u64,
    pub number_of_values: u64,
}
