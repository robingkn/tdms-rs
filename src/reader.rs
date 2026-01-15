use crate::error::{Result, TdmsError};
use crate::segment::Segment;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};

pub struct TdmsReader<R: Read + Seek> {
    reader: R,
    active_meta: std::collections::HashMap<String, crate::metadata::RawDataMeta>,
    object_order: Vec<String>,
}

impl<R: Read + Seek> TdmsReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            active_meta: std::collections::HashMap::new(),
            object_order: Vec::new(),
        }
    }

    pub fn read_segment(&mut self) -> Result<Segment> {
        let start_pos = self.reader.stream_position()?;

        // Read Lead In
        let mut lead_in = [0u8; 4];
        self.reader.read_exact(&mut lead_in)?;

        if &lead_in != b"TDSm" {
            return Err(TdmsError::InvalidSignature);
        }

        // Read Mask
        let mask = self.reader.read_u32::<LittleEndian>()?;

        // Read Version
        let version = self.reader.read_u32::<LittleEndian>()?;

        // Read Offsets
        let next_segment_offset = self.reader.read_u64::<LittleEndian>()?;
        let raw_data_offset = self.reader.read_u64::<LittleEndian>()?;

        let mask = crate::segment::Mask::new(mask);

        let mut objects = Vec::new();

        if mask.has_new_obj_list() {
            // Read number of objects
            let count = self.reader.read_u32::<LittleEndian>()?;
            self.object_order.clear();

            for _ in 0..count {
                // Read Object Path (String)
                let path_len = self.reader.read_u32::<LittleEndian>()?;
                let mut path_bytes = vec![0u8; path_len as usize];
                self.reader.read_exact(&mut path_bytes)?;
                let path_str =
                    String::from_utf8(path_bytes).map_err(|_| TdmsError::StringEncoding)?;

                self.object_order.push(path_str.clone());

                // Read Raw Data Index
                let raw_data_index = self.reader.read_u32::<LittleEndian>()?;

                let mut raw_data_meta = None;
                let prop_count;

                if raw_data_index != 0 && raw_data_index != 0xFFFFFFFF {
                    let mut skipped = vec![0u8; raw_data_index as usize];
                    self.reader.read_exact(&mut skipped)?;

                    if raw_data_index >= 4 {
                        let mut slice = &skipped[0..4];
                        let type_code = slice.read_u32::<LittleEndian>()?;
                        let data_type = crate::datatypes::DataType::from_u32(type_code)?;

                        // Parse common fields
                        let mut _dim = 1;
                        let mut count = 0;
                        let mut total_size = None;

                        if raw_data_index >= 8 {
                            let mut dim_slice = &skipped[4..8];
                            _dim = dim_slice.read_u32::<LittleEndian>()?;
                        }

                        if data_type == crate::datatypes::DataType::String {
                            // String Structure: Type(4), Dim(4), Count(8), TotalSize(8)
                            if raw_data_index >= 16 {
                                let mut count_slice = &skipped[8..16];
                                count = count_slice.read_u64::<LittleEndian>()?;
                            }
                            if raw_data_index >= 24 {
                                let mut size_slice = &skipped[16..24];
                                total_size = Some(size_slice.read_u64::<LittleEndian>()?);
                            } else if raw_data_index >= 20 {
                                // Some implementations (nptdms/LabVIEW?) write 20 bytes for String info
                                // Type(4) + Dim(4) + Count(8) + Size(4) = 20
                                let mut size_slice = &skipped[16..20];
                                total_size = Some(size_slice.read_u32::<LittleEndian>()? as u64);
                            }
                            // PropCount follows
                            prop_count = self.reader.read_u32::<LittleEndian>()?;
                        } else {
                            // Float/Numeric Structure: Type(4), Dim(4), Count(8). Index=20 (includes PropCount).
                            if raw_data_index >= 16 {
                                let mut count_slice = &skipped[8..16];
                                count = count_slice.read_u64::<LittleEndian>()?;
                            }

                            // Extact PropCount from last 4 bytes
                            let start = (raw_data_index - 4) as usize;
                            let mut end_slice = &skipped[start..];
                            prop_count = end_slice.read_u32::<LittleEndian>()?;
                        }

                        raw_data_meta = Some(crate::metadata::RawDataMeta {
                            data_type,
                            number_of_values: count,
                            total_size_bytes: total_size,
                        });
                    } else {
                        prop_count = 0;
                    }
                } else {
                    // No raw data info, PropCount follows directly
                    prop_count = self.reader.read_u32::<LittleEndian>()?;
                }

                // Read Properties

                let mut properties = std::collections::HashMap::new();

                for _ in 0..prop_count {
                    // Key
                    let key_len = self.reader.read_u32::<LittleEndian>()?;
                    let mut key_bytes = vec![0u8; key_len as usize];
                    self.reader.read_exact(&mut key_bytes)?;
                    let key =
                        String::from_utf8(key_bytes).map_err(|_| TdmsError::StringEncoding)?;

                    // Type
                    let type_code = self.reader.read_u32::<LittleEndian>()?;

                    // Value
                    let val = crate::datatypes::read_property_value(&mut self.reader, type_code)?;
                    properties.insert(key, val);
                }

                objects.push(crate::metadata::ParsingMetadata {
                    path: crate::metadata::ObjectPath::new(path_str),
                    raw_data_index,
                    properties,
                    raw_data_meta,
                    data_location: None,
                });
            }
        } else {
            // Reuse object list if it exists and has raw data
            for path_str in &self.object_order {
                objects.push(crate::metadata::ParsingMetadata {
                    path: crate::metadata::ObjectPath::new(path_str.clone()),
                    raw_data_index: 0,
                    properties: std::collections::HashMap::new(),
                    raw_data_meta: None,
                    data_location: None,
                });
            }
        }

        // Parse Raw Data
        // RawDataOffset is relative to the segment payload (Start + 28).
        // Dump shows: RawDataOffset = 93.
        // Header: 28 bytes.
        // 28 + 93 = 121 (0x79).
        // Dump at 0x79: 05 00 00 00 (Offset 5).
        // This is correct.

        let mut current_raw_offset = start_pos + 28 + raw_data_offset;

        for obj in &mut objects {
            let path_str = obj.path.raw.clone();

            if let Some(meta) = &obj.raw_data_meta {
                // Update active meta for this channel
                self.active_meta.insert(path_str.clone(), meta.clone());

                if meta.number_of_values > 0 {
                    // Record location instead of reading
                    let size = if let Some(s) = meta.data_type.size_of() {
                        s * meta.number_of_values
                    } else {
                        // String or Void
                        meta.total_size_bytes.unwrap_or(0)
                    };

                    obj.data_location = Some(crate::metadata::DataLocation {
                        offset: current_raw_offset,
                        number_of_values: meta.number_of_values,
                        data_type: meta.data_type.clone(),
                        total_size_bytes: meta.total_size_bytes,
                    });

                    current_raw_offset += size;
                }
            } else if obj.raw_data_index == 0 {
                // Use cached meta if available
                if let Some(meta) = self.active_meta.get(&path_str) {
                    if meta.number_of_values > 0 {
                        let size = if let Some(s) = meta.data_type.size_of() {
                            s * meta.number_of_values
                        } else {
                            // String or Void
                            meta.total_size_bytes.unwrap_or(0)
                        };

                        obj.data_location = Some(crate::metadata::DataLocation {
                            offset: current_raw_offset,
                            number_of_values: meta.number_of_values,
                            data_type: meta.data_type.clone(),
                            total_size_bytes: meta.total_size_bytes,
                        });

                        current_raw_offset += size;
                    }
                }
            }
        }

        let target_pos = if next_segment_offset != 0xFFFFFFFFFFFFFFFF {
            start_pos + 28 + next_segment_offset
        } else {
            current_raw_offset
        };

        let current_pos = self.reader.stream_position()?;
        if current_pos != target_pos {
            self.reader.seek(std::io::SeekFrom::Start(target_pos))?;
        }

        Ok(Segment {
            version,
            next_segment_offset,
            raw_data_offset,
            toc_mask: mask.convert(), // Need to export/convert mask logic or use u32
            objects,
        })
    }
}
