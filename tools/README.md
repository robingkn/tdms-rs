# Development Tools

Python scripts for TDMS corpus generation and debugging.

## Requirements
- Python 3.7+
- nptdms library: `pip install nptdms`

## Scripts
- `generate_corpus.py` - Generate test TDMS files
- `generate_json.py` - Create golden reference JSON
- `validate_json.py` - Verify JSON/TDMS consistency
- `debug_hex.py` - Hex dump TDMS files
- `debug_props.py` - Extract TDMS properties

## Usage
```bash
cd tools/
python generate_corpus.py
python validate_json.py
```

## Note
These tools are for development only. The Rust crate has no Python dependencies.