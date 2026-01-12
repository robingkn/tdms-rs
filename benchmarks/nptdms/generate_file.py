import numpy as np
from nptdms import TdmsWriter, ChannelObject, RootObject, GroupObject
import os

FILE_NAME = "benchmark_data.tdms"
TOTAL_SIZE_MB = 512
BYTES_PER_FLOAT = 8
NUM_ELEMENTS = (TOTAL_SIZE_MB * 1024 * 1024) // BYTES_PER_FLOAT

def generate_file():
    print(f"Generating ~{TOTAL_SIZE_MB} MB float64 TDMS file: {FILE_NAME}...")
    
    # Generate random data
    data = np.random.rand(NUM_ELEMENTS)
    
    root_object = RootObject(properties={
        "name": "BenchmarkFile",
        "description": "File for raw disk vs nptdms benchmark"
    })
    
    group_object = GroupObject("Group1", properties={})
    
    channel_object = ChannelObject(
        "Group1", "Channel1", data, properties={}
    )
    
    with TdmsWriter(FILE_NAME) as tdms_writer:
        tdms_writer.write_segment([
            root_object,
            group_object,
            channel_object
        ])
        
    print(f"Successfully generated {FILE_NAME}")
    print(f"File size: {os.path.getsize(FILE_NAME) / (1024*1024):.2f} MB")

if __name__ == "__main__":
    generate_file()
