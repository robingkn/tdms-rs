# TDMS Test Corpus

This directory contains the golden reference files used for integration and correctness testing.

## Regenerating Fixtures

If you need to update or regenerate these files, use the Python scripts provided in the `tools/` directory.

### Requirements
- Python 3.7+
- `nptdms` library: `pip install nptdms`

### Steps
1. Navigate to the `tools/` directory:
   ```bash
   cd tools/
   ```
2. Run the corpus generator:
   ```bash
   python generate_corpus.py
   ```
3. (Optional) Validate the generated files against their expected JSON representation:
   ```bash
   python validate_json.py
   ```

## Structure
- `01_minimal/`: Basic single-channel files.
- `03_datatypes/`: Coverage for all supported TDMS types.
- `06_properties/`: Nested metadata scenarios.
- ... and so on.

The Rust tests in `tests/golden_tests.rs` iterate through these directories to ensure `tdms-rs` can parse them correctly and match the expected values.
