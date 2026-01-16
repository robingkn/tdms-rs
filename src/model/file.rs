use crate::model::datatypes::PropertyValue;
use crate::model::group::TdmsGroupData;
use indexmap::IndexMap;
use std::path::PathBuf;

pub struct TdmsFileInner {
    pub path: PathBuf,
    pub groups: IndexMap<String, TdmsGroupData>,
    pub properties: IndexMap<String, PropertyValue>,
}
