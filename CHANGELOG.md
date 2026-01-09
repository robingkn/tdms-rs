# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-09

### Added
- **File-Level Property Support**: Complete implementation of file-level property reading and writing
- **Timestamp Property Handling**: Full support for TDMS timestamp properties with (i64, u64) format
- **Error-Path Test Coverage**: Comprehensive error handling tests for malformed files
- **Enhanced Documentation**: Added TDMS format guarantees, memory behavior, and performance characteristics
- **Writer API Clarification**: Documented channel data requirements and output guarantees

### Fixed
- **Eliminated All Compiler Warnings**: Removed unused imports, variables, and unreachable patterns
- **Binary Tool Accuracy**: Updated `tdms-to-json` to accurately describe its validation functionality
- **Property Round-Trip**: Added tests ensuring file-level properties survive write → read cycles

### Changed
- **Version 1.0.0**: Production-ready release with stable API
- **Enhanced README**: Added conceptual documentation and performance transparency
- **Improved Error Messages**: Better error reporting for common failure cases

## [0.1.0] - 2026-01-09

### Added

#### TDMS Reading Support
- Support for all TDMS data types (integers, floats, strings, timestamps, booleans)
- Command-line tool `tdms-to-json` for converting TDMS files to JSON
- Comprehensive test suite with 24 test cases covering edge cases
- Support for special float values (NaN, Infinity, -0.0)
- Unicode support for group and channel names
- Multi-segment file support
- Property metadata parsing at all levels (file, group, channel)

#### TDMS Writing Support ✨ NEW
- Complete TDMS file writer API with corpus-compatible output
- Support for all data types: Double, Float, I8-I64, U8-U64, Boolean, String, TimeStamp
- Multi-group, multi-channel file creation
- Properties support at file, group, and channel levels
- Deterministic channel ordering using BTreeMap
- Round-trip verification with existing corpus files
- TDD-developed with comprehensive test coverage

#### Examples and Documentation
- Four comprehensive writer examples:
  - `write_minimal.rs` - Basic TDMS file creation
  - `write_multi_channel.rs` - Multiple data types and channels
  - `write_properties.rs` - Properties at all levels
  - `write_all_types.rs` - Complete data type showcase
- Updated README with writer usage examples
- Complete API documentation with examples

### Features
- Zero-copy parsing where possible for reading
- Pure Rust implementation with no external dependencies
- Handles large and sparse data efficiently
- Support for incremental writes and append mode files (reading)
- Corpus-compatible writer output verified by round-trip testing