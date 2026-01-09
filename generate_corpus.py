
import os
import shutil
import numpy as np
from nptdms import TdmsWriter, RootObject, GroupObject, ChannelObject, TdmsFile
from datetime import datetime, timedelta, timezone
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

CORPUS_DIR = "tdms_corpus"

def clean_corpus_dir():
    if os.path.exists(CORPUS_DIR):
        shutil.rmtree(CORPUS_DIR)
    os.makedirs(CORPUS_DIR)
    logger.info(f"Cleaned and created {CORPUS_DIR}")

GENERATORS = []

def register_generator(func):
    GENERATORS.append(func)
    return func

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

# --- 01 Minimal ---
@register_generator
def generate_minimal():
    """Single channel, small data."""
    folder = os.path.join(CORPUS_DIR, "01_minimal")
    ensure_dir(folder)
    
    path = os.path.join(folder, "minimal.tdms")
    with TdmsWriter(path) as tdms_writer:
        channel = ChannelObject("Group", "Channel1", [1.1, 2.2, 3.3])
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 02 Structure Variants ---
@register_generator
def generate_structure_variants():
    folder = os.path.join(CORPUS_DIR, "02_structure_variants")
    ensure_dir(folder)

    # 1. Multiple segments
    path = os.path.join(folder, "multiple_segments.tdms")
    with TdmsWriter(path) as tdms_writer:
        # Segment 1
        curr_time = datetime.now(timezone.utc)
        root_obj = RootObject(properties={"description": "Segment 1"})
        channel = ChannelObject("Group", "Channel1", [1, 2, 3])
        tdms_writer.write_segment([root_obj, channel])
        
        # Segment 2
        root_obj = RootObject(properties={"description": "Segment 2"})
        channel = ChannelObject("Group", "Channel1", [4, 5, 6])
        tdms_writer.write_segment([root_obj, channel])
    logger.info(f"Generated {path}")

    # 2. Empty segment (metadata only update, no new data)
    path = os.path.join(folder, "empty_segment.tdms")
    with TdmsWriter(path) as tdms_writer:
        channel = ChannelObject("Group", "Channel1", [1, 2, 3])
        tdms_writer.write_segment([channel])
        
        # Empty segment writing only metadata
        root_obj = RootObject(properties={"updated": "true"})
        tdms_writer.write_segment([root_obj])
    logger.info(f"Generated {path}")

    # 3. Metadata Only - No Data
    path = os.path.join(folder, "metadata_only.tdms")
    with TdmsWriter(path) as tdms_writer:
        root_obj = RootObject(properties={"type": "metadata_only"})
        group_obj = GroupObject("Group1", properties={"desc": "An empty group"})
        # Channel with empty data
        channel_obj = ChannelObject("Group1", "Channel1", [])
        tdms_writer.write_segment([root_obj, group_obj, channel_obj])
    logger.info(f"Generated {path}")

    # 4. Root Only
    path = os.path.join(folder, "root_only.tdms")
    with TdmsWriter(path) as tdms_writer:
        root_obj = RootObject(properties={"title": "Root Only File"})
        tdms_writer.write_segment([root_obj])
    logger.info(f"Generated {path}")
    
    # 5. Group Only
    path = os.path.join(folder, "group_only.tdms")
    with TdmsWriter(path) as tdms_writer:
        group_obj = GroupObject("GroupOnly", properties={"desc": "No channels here"})
        tdms_writer.write_segment([group_obj])
    logger.info(f"Generated {path}")

# --- 03 Datatypes ---
@register_generator
def generate_datatypes():
    folder = os.path.join(CORPUS_DIR, "03_datatypes")
    ensure_dir(folder)

    # 1. Integers
    path = os.path.join(folder, "integers.tdms")
    with TdmsWriter(path) as tdms_writer:
        data_int8 = np.array([-128, -1, 0, 1, 127], dtype=np.int8)
        data_int16 = np.array([-32768, -1, 0, 1, 32767], dtype=np.int16)
        data_int32 = np.array([-2147483648, -1, 0, 1, 2147483647], dtype=np.int32)
        data_int64 = np.array([-9223372036854775808, -1, 0, 1, 9223372036854775807], dtype=np.int64)
        
        data_uint8 = np.array([0, 1, 255], dtype=np.uint8)
        data_uint16 = np.array([0, 1, 65535], dtype=np.uint16)
        data_uint32 = np.array([0, 1, 4294967295], dtype=np.uint32)
        data_uint64 = np.array([0, 1, 18446744073709551615], dtype=np.uint64)

        channels = [
            ChannelObject("Integers", "Int8", data_int8),
            ChannelObject("Integers", "Int16", data_int16),
            ChannelObject("Integers", "Int32", data_int32),
            ChannelObject("Integers", "Int64", data_int64),
            ChannelObject("Unsigned", "Uint8", data_uint8),
            ChannelObject("Unsigned", "Uint16", data_uint16),
            ChannelObject("Unsigned", "Uint32", data_uint32),
            ChannelObject("Unsigned", "Uint64", data_uint64),
        ]
        tdms_writer.write_segment(channels)
    logger.info(f"Generated {path}")

    # 2. Floats
    path = os.path.join(folder, "floats.tdms")
    with TdmsWriter(path) as tdms_writer:
        data_float32 = np.array([0.0, -1.0, 1.0, 3.14159, 1.23e-10], dtype=np.float32)
        data_float64 = np.array([0.0, -1.0, 1.0, 3.1415926535, 1.23e-20], dtype=np.float64)
        
        channels = [
            ChannelObject("Floats", "Float32", data_float32),
            ChannelObject("Floats", "Float64", data_float64),
        ]
        tdms_writer.write_segment(channels)
    logger.info(f"Generated {path}")
    
    # 3. Booleans
    path = os.path.join(folder, "booleans.tdms")
    with TdmsWriter(path) as tdms_writer:
        data_bool = [True, False, True, True, False, False]
        channel = ChannelObject("Booleans", "Flags", data_bool)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

    # 4. Strings
    path = os.path.join(folder, "strings.tdms")
    with TdmsWriter(path) as tdms_writer:
        # Note: nptdms might treat simple lists of strings as ... strings.
        data_str = ["Hello", "World", "", "TDMS", "File Format"]
        channel = ChannelObject("Strings", "Basic", data_str)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

    # 5. Timestamps (Basic)
    path = os.path.join(folder, "timestamps.tdms")
    with TdmsWriter(path) as tdms_writer:
        now = datetime.now(timezone.utc)
        data_time = [
            now,
            now + timedelta(seconds=1),
            now - timedelta(days=365),
            datetime(1904, 1, 1, tzinfo=timezone.utc)
        ]
        channel = ChannelObject("Time", "Events", data_time)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 04 Numeric Limits ---
@register_generator
def generate_numeric_limits():
    folder = os.path.join(CORPUS_DIR, "04_numeric_limits")
    ensure_dir(folder)

    # 1. Special Floats
    path = os.path.join(folder, "special_floats.tdms")
    with TdmsWriter(path) as tdms_writer:
        data_float = np.array([np.inf, -np.inf, np.nan, -0.0, 0.0], dtype=np.float64)
        channel = ChannelObject("Limits", "SpecialFloats", data_float)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 05 String Edge Cases ---
@register_generator
def generate_string_edge_cases():
    folder = os.path.join(CORPUS_DIR, "05_string_edge_cases")
    ensure_dir(folder)

    path = os.path.join(folder, "edge_cases.tdms")
    with TdmsWriter(path) as tdms_writer:
        long_string = "A" * 10000
        null_byte_string = "Null\x00Byte"
        unicode_string = "Hello \u00A9 \U0001F600"
        
        data = [long_string, null_byte_string, unicode_string]
        channel = ChannelObject("Strings", "Detailed", data)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 06 Properties ---
@register_generator
def generate_properties():
    folder = os.path.join(CORPUS_DIR, "06_properties")
    ensure_dir(folder)

    # 1. All Levels
    path = os.path.join(folder, "all_levels.tdms")
    with TdmsWriter(path) as tdms_writer:
        root_props = {
            "author": "Antigravity",
            "version": 1.0,
            "is_valid": True,
            "date": datetime.now(timezone.utc)
        }
        root_obj = RootObject(properties=root_props)
        
        group_props = {"department": "Test Engineering", "id": 42}
        
        channel_props = {"units": "Volts", "max_val": 10.5, "sensor": "A1"}
        channel = ChannelObject("Group", "Channel", [1.0, 2.0], properties=channel_props)
        group_obj = GroupObject("Group", properties=group_props)
        
        tdms_writer.write_segment([root_obj, group_obj, channel])
    logger.info(f"Generated {path}")

    # 2. Key Types
    path = os.path.join(folder, "property_keys.tdms")
    with TdmsWriter(path) as tdms_writer:
        props = {
            "standard": "value",
            "with spaces": "value",
            "with/slash": "value",
            "with.dot": "value",
            "unicode_\u03A9": "Ohm"
        }
        root_obj = RootObject(properties=props)
        tdms_writer.write_segment([root_obj])
    logger.info(f"Generated {path}")

# --- 07 Timestamps ---
@register_generator
def generate_timestamps_advanced():
    folder = os.path.join(CORPUS_DIR, "07_timestamps")
    ensure_dir(folder)
    
    path = os.path.join(folder, "high_precision.tdms")
    with TdmsWriter(path) as tdms_writer:
        t0 = datetime.now(timezone.utc)
        data = [
            t0,
            t0 + timedelta(microseconds=1),
            t0 + timedelta(microseconds=100),
            t0 + timedelta(milliseconds=1),
        ]
        channel = ChannelObject("Time", "SubSecond", data)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

    path = os.path.join(folder, "extreme_range.tdms")
    with TdmsWriter(path) as tdms_writer:
        t_epoch = datetime(1904, 1, 1, tzinfo=timezone.utc)
        t_past = datetime(1800, 1, 1, tzinfo=timezone.utc) 
        t_future = datetime(3000, 1, 1, tzinfo=timezone.utc)
        
        data = [t_epoch, t_past, t_future]
        channel = ChannelObject("Time", "Extremes", data)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 08 Raw vs Interleaved ---
@register_generator
def generate_raw_variants():
    folder = os.path.join(CORPUS_DIR, "08_raw_vs_interleaved")
    ensure_dir(folder)
    
    path = os.path.join(folder, "standard_layout.tdms")
    with TdmsWriter(path) as tdms_writer:
        c1 = ChannelObject("Group", "C1", np.arange(100, dtype=np.int32))
        c2 = ChannelObject("Group", "C2", np.arange(100, dtype=np.int32))
        tdms_writer.write_segment([c1, c2])
    logger.info(f"Generated {path}")

# --- 09 Scaling and Units ---
@register_generator
def generate_scaling():
    folder = os.path.join(CORPUS_DIR, "09_scaling_and_units")
    ensure_dir(folder)
    
    path = os.path.join(folder, "linear_scaling.tdms")
    with TdmsWriter(path) as tdms_writer:
        props = {
            "wf_start_offset": 10.0,
            "wf_increment": 0.5,
            "wf_start_time": datetime.now(timezone.utc),
            "NI_UnitDescription": "Volts",
            "unit_string": "V"
        }
        data = np.array([0, 1, 2, 3, 4], dtype=np.float64)
        channel = ChannelObject("Scaling", "Linear", data, properties=props)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 10 Large and Sparse ---
@register_generator
def generate_large_sparse():
    folder = os.path.join(CORPUS_DIR, "10_large_and_sparse")
    ensure_dir(folder)
    
    path = os.path.join(folder, "sparse.tdms")
    with TdmsWriter(path) as tdms_writer:
        size = 100000 
        data = np.zeros(size, dtype=np.float32)
        data[0] = 1.0
        data[-1] = 1.0
        channel = ChannelObject("Sparse", "MostlyZeros", data)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 11 Incremental Writes ---
@register_generator
def generate_incremental():
    folder = os.path.join(CORPUS_DIR, "11_incremental_writes")
    ensure_dir(folder)
    
    path = os.path.join(folder, "append_mode.tdms")
    # First write
    with TdmsWriter(path) as tdms_writer:
        channel = ChannelObject("Group", "Channel1", [1, 2, 3])
        tdms_writer.write_segment([channel])
    
    # Append
    with TdmsWriter(path, mode='a') as tdms_writer:
        channel = ChannelObject("Group", "Channel1", [4, 5, 6])
        tdms_writer.write_segment([channel])
        
    logger.info(f"Generated {path}")

# --- 12 Metadata Only ---
@register_generator
def generate_metadata_variants():
    folder = os.path.join(CORPUS_DIR, "12_metadata_only")
    ensure_dir(folder)
    
    path = os.path.join(folder, "no_data.tdms")
    with TdmsWriter(path) as tdms_writer:
        # Same as structure variant but in specific folder
        root_obj = RootObject(properties={"type": "metadata_only"})
        channel_obj = ChannelObject("Group1", "Channel1", [])
        tdms_writer.write_segment([root_obj, channel_obj])
    logger.info(f"Generated {path}")

# --- 13 Unicode and Encoding ---
@register_generator
def generate_unicode_paths():
    folder = os.path.join(CORPUS_DIR, "13_unicode_and_encoding")
    ensure_dir(folder)
    
    path = os.path.join(folder, "unicode_paths.tdms")
    with TdmsWriter(path) as tdms_writer:
        # Unicode in Group and Channel names
        group_name = "Gr\u00F6up_\u03A9" # Group_Omega
        channel_name = "Ch\u00E5nnel_\u2126" # Channel_Ohm
        
        data = [1, 2, 3]
        channel = ChannelObject(group_name, channel_name, data)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

# --- 14 Alignment and Padding ---
@register_generator
def generate_alignment():
    folder = os.path.join(CORPUS_DIR, "14_alignment_and_padding")
    ensure_dir(folder)
    
    path = os.path.join(folder, "odd_sizes.tdms")
    with TdmsWriter(path) as tdms_writer:
        # Writing odd number of bytes might test padding logic in parser
        # Boolean is 1 byte. 
        data = [True, False, True] # 3 bytes
        channel = ChannelObject("Group", "Bool3", data)
        tdms_writer.write_segment([channel])
    logger.info(f"Generated {path}")

if __name__ == "__main__":
    clean_corpus_dir()
    for gen in GENERATORS:
        try:
            gen()
        except Exception as e:
            logger.error(f"Failed to run generator {gen.__name__}: {e}")
            raise
    logger.info("Corpus generation complete.")
