
from nptdms import TdmsFile
import sys

path = "tdms_corpus/01_minimal/minimal.tdms"

with TdmsFile.read(path) as f:
    print(f"File Properties: {f.properties}")
    for group in f.groups():
        print(f"Group: {group.name}")
        print(f"  Properties: {group.properties}")
        for channel in group.channels():
            print(f"  Channel: {channel.name}")
            print(f"    Properties: {channel.properties}")
            # Access internal objects?
            # nptdms stores objects in .objects dict sometimes?
