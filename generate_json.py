
import os
import json
import math
import numpy as np
from nptdms import TdmsFile
import datetime

CORPUS_DIR = "tdms_corpus"

def tdms_timestamp_to_json(dt):
    """
    Convert datetime/numpy.datetime64 to TDMS {seconds, fraction} format.
    TDMS Epoch: 1904-01-01 00:00:00 UTC
    """
    if isinstance(dt, np.datetime64):
        # Convert to datetime (assuming UTC mostly, but numpy is timezone naive relative to epoch)
        # np.datetime64 is usually based on 1970 epoch
        # Let's convert to seconds since 1904
        
        # Epochs
        epoch_1904 = np.datetime64('1904-01-01T00:00:00')
        
        # Calculate difference in seconds
        # Using .item() converts to python datetime or int depending on unit?
        # Let's stay in numpy for precision if possible
        
        # Convert to microseconds first to handle standard resolution
        dt_us = dt.astype('datetime64[us]')
        diff = dt_us - epoch_1904.astype('datetime64[us]')
        
        # Total microseconds
        total_us = diff.astype(np.int64)
        
        # Seconds and remainder
        seconds = int(total_us // 1_000_000)
        remainder_us = int(total_us % 1_000_000)
        
        # Convert microseconds to 2^64 fractions
        # fraction = (us / 1_000_000) * 2^64
        fraction = int((remainder_us / 1_000_000.0) * (2**64))
        
        return {"seconds": seconds, "fraction": fraction}
        
    elif isinstance(dt, datetime.datetime):
         # Python datetime
         epoch_1904 = datetime.datetime(1904, 1, 1, tzinfo=datetime.timezone.utc)
         if dt.tzinfo is None:
             dt = dt.replace(tzinfo=datetime.timezone.utc)
             
         diff = dt - epoch_1904
         total_seconds = int(diff.total_seconds())
         microseconds = dt.microsecond
         
         # total_seconds from timedelta includes the microseconds? No
         # diff.total_seconds() returns float.
         # Let's use clean arithmetic
         
         full_seconds = diff.days * 86400 + diff.seconds
         fraction = int((microseconds / 1_000_000.0) * (2**64))
         return {"seconds": full_seconds, "fraction": fraction}
         
    return str(dt)

def convert_value(val):
    """
    Convert a value to the required JSON representation.
    """
    # Numerics
    if isinstance(val, (float, np.floating)):
        if np.isnan(val):
            return "NaN"
        if np.isinf(val):
            if val > 0:
                return "Infinity"
            else:
                return "-Infinity"
        # Check for negative zero
        # Copysign: copysign(1.0, -0.0) -> -1.0
        if val == 0.0 and math.copysign(1.0, val) == -1.0:
            return "-0.0"
        return float(val)

    if isinstance(val, (int, np.integer)):
        return int(val)
        
    if isinstance(val, (bool, np.bool_)):
        return bool(val)

    if isinstance(val, (np.datetime64, datetime.datetime)):
        return tdms_timestamp_to_json(val)
        
    if isinstance(val, bytes):
        # Decode utf-8 strictly
        return val.decode('utf-8')
        
    if isinstance(val, str):
        return val
        
    if isinstance(val, list):
        return [convert_value(v) for v in val]
        
    if isinstance(val, np.ndarray):
        return [convert_value(v) for v in val.tolist()]

    return str(val)

def process_corpus():
    for root, dirs, files in os.walk(CORPUS_DIR):
        for file in files:
            if file.endswith(".tdms"):
                tdms_path = os.path.join(root, file)
                json_path = tdms_path.replace(".tdms", ".json")
                
                print(f"Processing {tdms_path}...")
                
                try:
                    with TdmsFile.read(tdms_path) as tdms_file:
                        json_content = {
                            "file_properties": {k: convert_value(v) for k, v in tdms_file.properties.items()},
                            "groups": {}
                        }
                        
                        for group in tdms_file.groups():
                            group_dict = {
                                "properties": {k: convert_value(v) for k, v in group.properties.items()},
                                "channels": {}
                            }
                            
                            for channel in group.channels():
                                # Determine dtype string
                                dtype_str = str(channel.dtype)
                                
                                # Get Data
                                try:
                                    # Accessing data might trigger reading
                                    data_raw = channel.data
                                    data_list = [convert_value(x) for x in data_raw]
                                except Exception as e:
                                    print(f"Error reading data for {channel.name}: {e}")
                                    data_list = []

                                channel_dict = {
                                    "dtype": dtype_str,
                                    "data": data_list,
                                    "properties": {k: convert_value(v) for k, v in channel.properties.items()}
                                }
                                group_dict["channels"][channel.name] = channel_dict
                            
                            json_content["groups"][group.name] = group_dict
                            
                        # Write JSON
                        with open(json_path, 'w', encoding='utf-8') as f:
                            json.dump(json_content, f, indent=2, sort_keys=True, ensure_ascii=False)
                            
                except Exception as e:
                    print(f"FAILED {tdms_path}: {e}")
                    raise

if __name__ == "__main__":
    process_corpus()
