# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-09

### Added
- **Complete TDMS Reading Support**: Full support for all TDMS data types and structures
  - All integer types: I8, I16, I32, I64, U8, U16, U32, U64
  - Floating point types: Float (f32), Double (f64)
  - String, Boolean, and TimeStamp data types
  - Special float values (NaN, Infinity, -0.0) fully supported
- **TDMS Writing Support**: Create TDMS files from Rust data structures
  - Multi-group, multi-channel file creation
  - Properties support at file, group, and channel levels
  - Deterministic channel ordering using BTreeMap
  - Binary compatibility with National Instruments TDMS readers
- **File-Level Property Support**: Complete implementation of file-level property reading and writing
- **Timestamp Property Handling**: Full support for TDMS timestamp properties with (i64, u64) format
- **Command-Line Tool**: `tdms-to-json` binary for file validation and structure inspection
- **Comprehensive Examples**: Four detailed examples demonstrating different use cases
  - `write_minimal.rs` - Basic TDMS file creation
  - `write_multi_channel.rs` - Multiple data types and channels
  - `write_properties.rs` - Properties at all levels
  - `write_all_types.rs` - Complete data type showcase
- **Enhanced Error Handling**: Robust error handling with descriptive error messages using `thiserror`
- **Test Coverage**: Comprehensive test suite with 24+ test scenarios covering edge cases

### Features
- **Zero-Copy Parsing**: Efficient memory usage where possible
- **Streaming Reads**: Handle large files without loading everything into memory
- **Pure Rust Implementation**: Minimal external dependencies
- **Binary Compatibility**: Output files verified against National Instruments corpus
- **Round-Trip Testing**: Write → read cycles preserve all data integrity
- **Unicode Support**: Full support for Unicode in group and channel names
- **Multi-Segment Support**: Read files with multiple segments and incremental writes
- **Property Metadata**: Complete support for properties at all levels (file, group, channel)
- **Ordered Collections**: Deterministic iteration order using IndexMap
- **Input Validation**: Writer API validates inputs and prevents invalid files
- **Convenience Methods**: Enhanced API with type-safe data access methods
- **Display Traits**: Human-readable formatting for debugging

### Performance
- **Memory Efficient**: Peak memory usage proportional to largest channel's data size
- **I/O Optimized**: Sequential reading minimizes disk seeks
- **Deterministic Output**: Consistent file generation with alphabetical ordering
- **Large File Support**: Efficient handling of large TDMS files through streaming

### Documentation
- **Complete API Documentation**: Rustdoc comments for all public APIs with examples
- **README with Examples**: Comprehensive usage examples and getting started guide
- **Format Guarantees**: Clear documentation of binary compatibility and behavior
- **Performance Characteristics**: Transparent documentation of memory and I/O behavior

### Dependencies
- `byteorder = "1.4"` - Efficient binary data parsing
- `ordered-float = "5.1.0"` - Handling of special float values
- `serde = "1.0"` - Serialization support for JSON tool
- `serde_json = "1.0"` - JSON output for command-line tool
- `thiserror = "1.0"` - Ergonomic error handling
- `indexmap = "2.0"` - Ordered collections for deterministic behavior

## [0.1.0] - 2026-01-09

### Added
- Initial development release
- Basic TDMS reading functionality
- Foundation for writer implementation

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