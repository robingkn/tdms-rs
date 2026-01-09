# TDMS Test Corpus

This directory contains a comprehensive set of TDMS files generated to test TDMS parsers, covering structural variations, datatypes, edge cases, and limits. The files are generated using the `nptdms` library.

## Directory Structure & Coverage

| Folder | Description | Key Files |
|--------|-------------|-----------|
| `01_minimal` | Basic sanity checks | `minimal.tdms` (Simple Group/Channel/Data) |
| `02_structure_variants` | File structure tests | `multiple_segments.tdms` (Fragmentation), `empty_segment.tdms` (Metadata updates), `metadata_only.tdms` (Structure w/o data) |
| `03_datatypes` | All supported simple datatypes | `integers.tdms`, `floats.tdms`, `booleans.tdms`, `strings.tdms` |
| `04_numeric_limits` | Boundary values | `special_floats.tdms` (NaN, Inf), `int_bounds.tdms` |
| `05_string_edge_cases` | Complex string scenarios | `edge_cases.tdms` (Long strings, null bytes, unicode) |
| `06_properties` | Property metadata at all levels | `all_levels.tdms` (Root/Group/Channel props), `property_keys.tdms` (Unicode/Space keys) |
| `07_timestamps` | Time and date handling | `high_precision.tdms` (Sub-second), `extreme_range.tdms` (Past/Future) |
| `08_raw_vs_interleaved` | Data layout variations | `standard_layout.tdms` |
| `09_scaling_and_units` | Data interpretation attributes | `linear_scaling.tdms` (wf_increment, wf_start_offset) |
| `10_large_and_sparse` | Performance stressors | `sparse.tdms` (Large zero arrays) |
| `11_incremental_writes` | Append workflows | `append_mode.tdms` (File opened in 'a' mode) |
| `12_metadata_only` | Structure without samples | `no_data.tdms` |
| `13_unicode_and_encoding` | Path encoding | `unicode_paths.tdms` (Non-ASCII Group/Channel names) |
| `14_alignment_and_padding` | Byte alignment checks | `odd_sizes.tdms` (Odd byte counts) |

## Golden Reference JSON (Rust Spec)
Each `.tdms` file has a corresponding `.json` file in the same directory. These JSON files serve as the canonical ground truth for testing parsers (specifically for Rust implementation).

### Encoding Rules
- **Numerics**: Strict encoding for `NaN`, `Infinity`, `-Infinity`, `-0.0` (as strings).
- **Timestamps**: Objects with `{seconds: <int>, fraction: <2^64 int>}` (TDMS 1904 epoch).
- **Determinism**: Keys sorted lexicographically, UTF-8 encoded.

### Validation
Run `python validate_json.py` to verify that all JSON files strictly match their TDMS counterparts (100% fidelity).

## Comparison with TDMS Spec
This corpus relies on `nptdms` implementation of the TDMS specification.
- **Timestamps**: Uses standard TDMS epoch (1904).
- **Strings**: UTF-8 encoded in newer formats.
- **Properties**: Supported on all object types.

## Generation
Run the generation script to recreate this corpus:
```bash
python generate_corpus.py
```
This will clean and repopulate the `tdms_corpus` directory.
