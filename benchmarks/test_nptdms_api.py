#!/usr/bin/env python3
"""
Test script to understand the nptdms API and fix compatibility issues.
"""

import numpy as np
import tempfile
import os

try:
    import nptdms
    from nptdms import TdmsWriter, ChannelObject
    print(f"nptdms version: {nptdms.__version__}")
except ImportError as e:
    print(f"Error importing nptdms: {e}")
    exit(1)

def test_basic_write():
    """Test basic TDMS writing functionality."""
    print("Testing basic TDMS write...")
    
    with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
        tmp_path = tmp_file.name
    
    try:
        # Test basic channel creation
        data = np.random.random(1000).astype(np.float64)
        channel = ChannelObject('TestGroup', 'TestChannel', data)
        
        print(f"Channel created: {channel}")
        print(f"Channel properties type: {type(channel.properties)}")
        print(f"Channel properties: {channel.properties}")
        
        # Try to set properties
        if channel.properties is None:
            print("Properties is None, creating new dict")
            channel.properties = {}
        
        channel.properties['test_prop'] = 'test_value'
        print(f"Properties after setting: {channel.properties}")
        
        # Test writing
        with TdmsWriter(tmp_path) as tdms_writer:
            print(f"TdmsWriter created: {tdms_writer}")
            
            # Check if file_properties exists
            if hasattr(tdms_writer, 'file_properties'):
                print("TdmsWriter has file_properties attribute")
                tdms_writer.file_properties = {'Title': 'Test File'}
            else:
                print("TdmsWriter does not have file_properties attribute")
            
            tdms_writer.write_data([channel])
            print("Data written successfully")
        
        # Test reading
        tdms_file = nptdms.TdmsFile.read(tmp_path)
        print(f"File read successfully: {tdms_file}")
        
        # Check groups and channels
        groups = list(tdms_file.groups())
        print(f"Groups: {[g.name for g in groups]}")
        
        if groups:
            channels = list(groups[0].channels())
            print(f"Channels in first group: {[c.name for c in channels]}")
            
            if channels:
                test_channel = channels[0]
                print(f"Test channel data length: {len(test_channel.data) if hasattr(test_channel, 'data') and test_channel.data is not None else 'No data'}")
                print(f"Test channel properties: {test_channel.properties if hasattr(test_channel, 'properties') else 'No properties'}")
        
        print("✅ Basic write/read test passed")
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
    
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)

def test_api_methods():
    """Test various nptdms API methods."""
    print("\nTesting API methods...")
    
    # Check ChannelObject constructor
    try:
        data = np.array([1.0, 2.0, 3.0])
        channel = ChannelObject('Group', 'Channel', data)
        print(f"✅ ChannelObject constructor works")
        print(f"   Channel: {channel}")
        print(f"   Properties: {channel.properties}")
        print(f"   Properties type: {type(channel.properties)}")
    except Exception as e:
        print(f"❌ ChannelObject constructor failed: {e}")
    
    # Check TdmsWriter
    try:
        with tempfile.NamedTemporaryFile(suffix='.tdms', delete=False) as tmp_file:
            tmp_path = tmp_file.name
        
        writer = TdmsWriter(tmp_path)
        print(f"✅ TdmsWriter constructor works")
        print(f"   Writer: {writer}")
        print(f"   Has file_properties: {hasattr(writer, 'file_properties')}")
        
        # Clean up
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)
            
    except Exception as e:
        print(f"❌ TdmsWriter constructor failed: {e}")

if __name__ == "__main__":
    print("nptdms API Test")
    print("=" * 30)
    
    test_api_methods()
    test_basic_write()