
with open("tdms_corpus/03_datatypes/strings.tdms", "rb") as f:
    data = f.read()

# Find "Basic"
pos = data.find(b"Basic")
if pos == -1:
    print("Basic not found")
else:
    print(f"Found Basic at {pos}")
    # Context
    # Path: /'Strings'/'Basic'
    # Start checking from pos-4 (PathLen) or full path.
    
    # Path len might not be near if I search for "Basic".
    # Just dump region.
    start = pos - 20
    chunk = data[start : start + 80]
    print("Hex Dump:")
    print(" ".join(f"{b:02X}" for b in chunk))
    # Context: PathLen(4) + "Channel1"(8) ...
    # We want to see bytes starting from "Channel1" end.
    start = pos + 8 # Channel1 length is 8
    # Actually path is longer /'Group'/'Channel1' (19 bytes)
    # The string "Channel1" is at the end of path.
    # Start checking from pos-4 (PathLen) or full path.
    
    # Path: /'Group'/'Channel1'
    # Length: 19 (0x13).
    # Find 0x13 00 00 00
    
    path_len_pos = data.find(b"\x13\x00\x00\x00")
    print(f"Found PathLen 19 at {path_len_pos}")
    
    if path_len_pos != -1:
        # Dump 64 bytes from there
        chunk = data[path_len_pos : path_len_pos + 64]
        print("Hex Dump:")
        print(" ".join(f"{b:02X}" for b in chunk))
        
        # Legend:
        # 0-3: Path Len (13 00 00 00)
        # 4-22: Path String (2F 27 47 ... )
        # 23-26: Raw Data Index (14 00 00 00 = 20)
        # 27-46: Raw Data Info (20 bytes)
        # 47-50: Prop Count (00 00 00 00)
        # 51..: Data (9A ...)
