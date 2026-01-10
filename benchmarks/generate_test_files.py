"""
Generate TDMS test files for benchmarking.

Creates a comprehensive set of TDMS files with known characteristics
for reproducible benchmarking. Files are designed to be reusable
by the Rust implementation for fair comparison.
"""

import numpy as np
from pathlib import Path
from typing import List, Dict, Any
import tempfile
import shutil

try:
    from nptdms import TdmsWriter, ChannelObject
except ImportError:
    print("Error: nptdms not installed. Run: pip install nptdms")
    exit(1)


def create_small_files(output_dir: Path):
    """Create small test files (1-10 MB)."""
    print("Creating small test files...")
    
    # Small single channel
    file_path = output_dir / "small_single_channel.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        data = np.random.random(10000).astype(np.float64)
        channel = ChannelObject('Group1', 'Channel1', data)
        channel.properties['wf_unit_string'] = 'V'
        channel.properties['wf_increment'] = 0.001
        tdms_writer.write_data([channel])
    
    # Small multi-channel
    file_path = output_dir / "small_multi_channel.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for i in range(5):
            data = np.random.random(5000).astype(np.float64)
            channel = ChannelObject('Sensors', f'Channel_{i}', data)
            channel.properties['Description'] = f'Test channel {i}'
            channels.append(channel)
        tdms_writer.write_data(channels)
    
    # Small with properties
    file_path = output_dir / "small_with_properties.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        # File properties
        tdms_writer.file_properties = {
            'Title': 'Benchmark Test File',
            'Author': 'nptdms Benchmark Suite',
            'Version': 1.0,
            'Sample_Rate': 1000.0
        }
        
        data = np.random.random(8000).astype(np.float64)
        channel = ChannelObject('Data', 'Temperature', data)
        channel.properties.update({
            'wf_unit_string': '°C',
            'wf_increment': 0.001,
            'Description': 'Temperature sensor data',
            'Calibration_Date': '2024-01-01',
            'Sensor_Type': 'Thermocouple'
        })
        tdms_writer.write_data([channel])
    
    # Small mixed data types
    file_path = output_dir / "small_mixed_types.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = [
            ChannelObject('Data', 'Doubles', np.random.random(2000).astype(np.float64)),
            ChannelObject('Data', 'Floats', np.random.random(2000).astype(np.float32)),
            ChannelObject('Data', 'Integers', np.random.randint(0, 1000, 2000).astype(np.int32)),
            ChannelObject('Data', 'Shorts', np.random.randint(0, 100, 2000).astype(np.int16)),
            ChannelObject('Data', 'Bytes', np.random.randint(0, 255, 2000).astype(np.uint8)),
        ]
        tdms_writer.write_data(channels)


def create_medium_files(output_dir: Path):
    """Create medium test files (100-500 MB)."""
    print("Creating medium test files...")
    
    # Medium single large channel
    file_path = output_dir / "medium_single_large.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        # ~100MB of float64 data
        data = np.random.random(12500000).astype(np.float64)
        channel = ChannelObject('Data', 'LargeChannel', data)
        channel.properties['Description'] = 'Large single channel for throughput testing'
        tdms_writer.write_data([channel])
    
    # Medium many channels
    file_path = output_dir / "medium_many_channels.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for i in range(50):
            data = np.random.random(250000).astype(np.float64)
            channel = ChannelObject('Sensors', f'Sensor_{i:03d}', data)
            channel.properties['Unit'] = 'V' if i % 2 == 0 else 'A'
            channel.properties['Range'] = f'{i * 10}-{(i + 1) * 10}'
            channels.append(channel)
        tdms_writer.write_data(channels)
    
    # Medium with multiple groups
    file_path = output_dir / "medium_multi_group.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        groups = ['Temperature', 'Pressure', 'Voltage', 'Current']
        
        for group in groups:
            for i in range(10):
                data = np.random.random(312500).astype(np.float64)  # ~25MB per group
                channel = ChannelObject(group, f'Channel_{i}', data)
                channel.properties['Group_Type'] = group
                channel.properties['Channel_Index'] = i
                channels.append(channel)
        
        tdms_writer.write_data(channels)


def create_large_files(output_dir: Path):
    """Create large test files (1-5 GB) - only in full mode."""
    print("Creating large test files...")
    
    # Large single massive channel
    file_path = output_dir / "large_single_massive.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        # ~1GB of float64 data
        data = np.random.random(125000000).astype(np.float64)
        channel = ChannelObject('Data', 'MassiveChannel', data)
        channel.properties['Description'] = 'Massive single channel for stress testing'
        tdms_writer.write_data([channel])
    
    # Large many medium channels
    file_path = output_dir / "large_many_medium.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for i in range(100):
            data = np.random.random(1250000).astype(np.float64)  # ~10MB each
            channel = ChannelObject('Data', f'Channel_{i:03d}', data)
            channel.properties['Index'] = i
            channels.append(channel)
        tdms_writer.write_data(channels)


def create_stress_files(output_dir: Path):
    """Create files for stress testing."""
    print("Creating stress test files...")
    
    # Many tiny channels
    file_path = output_dir / "stress_many_tiny.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for i in range(1000):
            data = np.random.random(10).astype(np.float64)  # Only 10 samples each
            channel = ChannelObject('Data', f'Tiny_{i:04d}', data)
            channels.append(channel)
        tdms_writer.write_data(channels)
    
    # Few channels with many properties
    file_path = output_dir / "stress_many_properties.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for i in range(5):
            data = np.random.random(10000).astype(np.float64)
            channel = ChannelObject('Data', f'Channel_{i}', data)
            
            # Add many properties
            for j in range(100):
                channel.properties[f'Property_{j:03d}'] = f'Value_{j}'
                channel.properties[f'Numeric_{j:03d}'] = j * 1.5
                channel.properties[f'Boolean_{j:03d}'] = j % 2 == 0
            
            channels.append(channel)
        tdms_writer.write_data(channels)
    
    # Mixed data types stress test
    file_path = output_dir / "stress_mixed_types.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        
        # Create channels of each data type
        data_types = [
            ('float64', np.float64, lambda n: np.random.random(n)),
            ('float32', np.float32, lambda n: np.random.random(n).astype(np.float32)),
            ('int32', np.int32, lambda n: np.random.randint(-1000, 1000, n).astype(np.int32)),
            ('int16', np.int16, lambda n: np.random.randint(-100, 100, n).astype(np.int16)),
            ('uint8', np.uint8, lambda n: np.random.randint(0, 255, n).astype(np.uint8)),
        ]
        
        for type_name, dtype, generator in data_types:
            for i in range(20):  # 20 channels of each type
                data = generator(5000)
                channel = ChannelObject('Mixed', f'{type_name}_{i:02d}', data)
                channel.properties['DataType'] = type_name
                channels.append(channel)
        
        tdms_writer.write_data(channels)


def create_structure_variants(output_dir: Path):
    """Create files with different structural patterns."""
    print("Creating structural variant files...")
    
    # Single group, many channels
    file_path = output_dir / "structure_single_group.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for i in range(100):
            data = np.random.random(1000).astype(np.float64)
            channel = ChannelObject('SingleGroup', f'Channel_{i:03d}', data)
            channels.append(channel)
        tdms_writer.write_data(channels)
    
    # Many groups, few channels each
    file_path = output_dir / "structure_many_groups.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        channels = []
        for group_i in range(50):
            for chan_i in range(2):
                data = np.random.random(1000).astype(np.float64)
                channel = ChannelObject(f'Group_{group_i:03d}', f'Channel_{chan_i}', data)
                channels.append(channel)
        tdms_writer.write_data(channels)
    
    # Metadata-only file
    file_path = output_dir / "structure_metadata_only.tdms"
    with TdmsWriter(str(file_path)) as tdms_writer:
        # File with properties but no data
        tdms_writer.file_properties = {
            'Title': 'Metadata Only File',
            'Description': 'File with extensive metadata but no channel data',
            'Created': '2024-01-01T00:00:00Z',
            'Version': '1.0.0'
        }
        
        # Create empty channels (metadata only)
        channels = []
        for i in range(10):
            # Empty data array
            data = np.array([], dtype=np.float64)
            channel = ChannelObject('Metadata', f'EmptyChannel_{i}', data)
            channel.properties.update({
                'Description': f'Empty channel {i}',
                'Unit': 'V',
                'Range': '0-10',
                'Calibrated': True
            })
            channels.append(channel)
        
        tdms_writer.write_data(channels)


def main(mode: str = 'full'):
    """Generate all benchmark test files."""
    output_dir = Path(__file__).parent / "test_files"
    output_dir.mkdir(exist_ok=True)
    
    print(f"Generating benchmark test files in {output_dir}")
    print(f"Mode: {mode}")
    
    # Always create these for smoke tests
    create_small_files(output_dir)
    create_structure_variants(output_dir)
    
    if mode == 'full':
        create_medium_files(output_dir)
        create_large_files(output_dir)
        create_stress_files(output_dir)
    
    # List generated files with sizes
    print("\nGenerated files:")
    total_size = 0
    for file_path in sorted(output_dir.glob("*.tdms")):
        size_mb = file_path.stat().st_size / 1024 / 1024
        total_size += size_mb
        print(f"  {file_path.name}: {size_mb:.1f} MB")
    
    print(f"\nTotal size: {total_size:.1f} MB")
    print("Test file generation complete!")


if __name__ == "__main__":
    import sys
    mode = sys.argv[1] if len(sys.argv) > 1 else 'full'
    main(mode)