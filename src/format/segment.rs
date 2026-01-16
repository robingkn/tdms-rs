use crate::format::metadata::ParsingMetadata;

#[derive(Debug)]
pub struct Segment {
    pub _version: u32,
    pub _next_segment_offset: u64,
    pub _raw_data_offset: u64,
    pub _toc_mask: u32,
    pub objects: Vec<ParsingMetadata>,
}

pub struct Mask {
    props: u32,
}

impl Mask {
    pub fn new(val: u32) -> Self {
        Self { props: val }
    }

    pub fn has_new_obj_list(&self) -> bool {
        (self.props & (1 << 2)) != 0
    }

    pub fn convert(&self) -> u32 {
        self.props
    }
}
