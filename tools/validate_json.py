
import os
import json
import numpy as np
import math
from nptdms import TdmsFile
import datetime

CORPUS_DIR = "tdms_corpus"

def compare_values(tdms_val, json_val, context=""):
    """
    Compare a value from nptdms with the value loaded from JSON.
    Returns True if match, False otherwise.
    """
    # Special floats
    if isinstance(tdms_val, (float, np.floating)):
        # Check against string representations in JSON if special
        if np.isnan(tdms_val):
            if json_val == "NaN":
                return True
            return False
        if np.isinf(tdms_val):
            if tdms_val > 0:
                if json_val == "Infinity":
                    return True
                return False
            else:
                if json_val == "-Infinity":
                    return True
                return False
        if tdms_val == 0.0 and math.copysign(1.0, tdms_val) == -1.0:
             if json_val == "-0.0":
                 return True
             # Allow numeric 0.0? No, strict encoding requested.
             return False

        # Normal float comparison
        # JSON loads as float usually, unless we parse strict? 
        # But we wrote simple floats as numbers.
        try:
            return math.isclose(tdms_val, float(json_val), rel_tol=1e-9)
        except ValueError:
            return False

    # Integers/Bools
    if isinstance(tdms_val, (int, np.integer, bool, np.bool_)):
         return tdms_val == json_val

    # Timestamps
    if isinstance(tdms_val, (np.datetime64, datetime.datetime)):
        # JSON is {seconds, fraction}
        if not isinstance(json_val, dict) or "seconds" not in json_val:
             return False
        
        # Convert TDMS val to expected dict
        # Can't easily reuse generation logic without importing it, let's duplicate strictly or check consistency
        # Rough check: convert json back to roughly datetime and compare?
        # Better: Strict check if possible.
        # Let's convert tdms_val to strict dict using same logic as generator (verification by regeneration-check)
        # This confirms generating logic consistency.
        
        # (Re-implementation of logic for verification)
        if isinstance(tdms_val, np.datetime64):
            dt_us = tdms_val.astype('datetime64[us]')
            epoch_1904 = np.datetime64('1904-01-01T00:00:00')
            total_us = (dt_us - epoch_1904.astype('datetime64[us]')).astype(np.int64)
            seconds = int(total_us // 1_000_000)
            fraction = int((int(total_us % 1_000_000) / 1_000_000.0) * (2**64))
        else: # datetime
             epoch_1904 = datetime.datetime(1904, 1, 1, tzinfo=datetime.timezone.utc)
             if tdms_val.tzinfo is None: tdms_val = tdms_val.replace(tzinfo=datetime.timezone.utc)
             diff = tdms_val - epoch_1904
             seconds = diff.days * 86400 + diff.seconds
             fraction = int((tdms_val.microsecond / 1_000_000.0) * (2**64))

        if seconds != json_val["seconds"]:
            # Tolerance for leap seconds? No, TDMS is simple seconds.
            return False
            
        # Fraction tolerance?
        # Re-calc can have minor precision noise? integer math should be stable
        if abs(fraction - json_val["fraction"]) > 1000: # generous tolerance for float-math noise if any
             return False
        return True

    # Strings/Bytes
    if isinstance(tdms_val, bytes):
        return tdms_val.decode('utf-8') == json_val
    
    return str(tdms_val) == str(json_val)

def validate_corpus():
    success = True
    for root, dirs, files in os.walk(CORPUS_DIR):
        for file in files:
            if file.endswith(".tdms"):
                tdms_path = os.path.join(root, file)
                json_path = tdms_path.replace(".tdms", ".json")
                
                if not os.path.exists(json_path):
                    print(f"MISSING JSON for {tdms_path}")
                    success = False
                    continue
                
                # Check JSON Schema Basics (loadability)
                try:
                    with open(json_path, 'r', encoding='utf-8') as f:
                        json_data = json.load(f)
                except Exception as e:
                    print(f"INVALID JSON {json_path}: {e}")
                    success = False
                    continue
                
                # Comparison
                try:
                    with TdmsFile.read(tdms_path) as tdms_file:
                        # File props
                        for k, v in tdms_file.properties.items():
                            if k not in json_data["file_properties"]:
                                print(f"Missing file property {k} in {json_path}")
                                success = False
                            elif not compare_values(v, json_data["file_properties"][k], f"File Prop {k}"):
                                print(f"Mismatch file property {k} in {json_path}. TDMS: {v}, JSON: {json_data['file_properties'][k]}")
                                success = False
                        
                        # Groups
                        for group in tdms_file.groups():
                            if group.name not in json_data["groups"]:
                                print(f"Missing group {group.name} in {json_path}")
                                success = False
                                continue
                            
                            g_json = json_data["groups"][group.name]
                            
                            # Group Props
                            for k, v in group.properties.items():
                                if k not in g_json["properties"]:
                                    print(f"Missing group property {k} in {json_path} [{group.name}]")
                                    success = False
                                elif not compare_values(v, g_json["properties"][k], f"Group {group.name} Prop {k}"):
                                    print(f"Mismatch group property {k} in {json_path}")
                                    success = False
                                    
                            # Channels
                            for channel in group.channels():
                                if channel.name not in g_json["channels"]:
                                    print(f"Missing channel {channel.name} in {json_path} [{group.name}]")
                                    success = False
                                    continue
                                
                                c_json = g_json["channels"][channel.name]
                                
                                # Data
                                tdms_data = channel.data
                                json_data_arr = c_json["data"]
                                
                                if len(tdms_data) != len(json_data_arr):
                                    print(f"Length mismatch {channel.name} in {json_path}")
                                    success = False
                                else:
                                    for i, (tv, jv) in enumerate(zip(tdms_data, json_data_arr)):
                                        if not compare_values(tv, jv, f"Ch {channel.name} idx {i}"):
                                            print(f"Data Mismatch {channel.name} at {i}. TDMS: {tv}, JSON: {jv}")
                                            success = False
                                            break # Stop after first mismatch per channel to avoid spam

                except Exception as e:
                    print(f"Error validating {tdms_path}: {e}")
                    import traceback
                    traceback.print_exc()
                    success = False
    
    if success:
        print("Validation Successful: All JSON files match TDMS.")
    else:
        print("Validation FAILED.")
        exit(1)

if __name__ == "__main__":
    validate_corpus()
