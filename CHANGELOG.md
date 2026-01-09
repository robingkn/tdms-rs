# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-01-09

### Added
- **Enhanced Error Types**: Expanded `TdmsError` enum with specific variants for better error handling
  - `GroupNotFound`, `ChannelNotFound`, `UnsupportedDataType`
  - `InvalidName`, `DuplicateName` for input validation
  - `InvalidFormat` for malformed TDMS files
- **Input Validation**: Writer API now validates names and prevents duplicates
  - Empty group/channel/property names are rejected with clear errors
  - Duplicate groups and channels within groups are prevented
  - All validation errors include descriptive messages
- **Convenience Methods for Data Access**: New typed accessor methods on `TdmsChannel`
  - `as_f64()`, `as_f32()`, `as_i32()`, `as_string()` for direct type access
  - `as_numeric()` converts any numeric type to `Vec<f64>`
  - `data_len()`, `data_type_name()` for metadata access
  - Property helpers: `unit()`, `increment()`, `start_time()`
  - Generic property getters: `get_string_property()`, `get_double_property()`, `get_i32_property()`
- **Utility Methods for TdmsData**: Enhanced data manipulation capabilities
  - `len()`, `is_empty()`, `type_name()`, `is_numeric()` methods
  - Consistent API across all data types
- **Display Trait Implementations**: Human-readable formatting for all types
  - `PropertyValue` displays with proper formatting (strings quoted, special float values)
  - `TdmsData` shows type and length: "Double [100]"
  - Special handling for NaN, ±∞ in floating-point values
- **Ordered Collections**: Replaced `HashMap` with `IndexMap` for deterministic ordering
  - File and group properties maintain insertion order
  - Groups maintain insertion order in files
  - Channels use `BTreeMap` for alphabetical ordering within groups
  - New iterator methods: `iter_groups()`, `iter_channels()`
- **Default and Constructor Implementations**: Standard Rust patterns
  - `Default` implementations for `TdmsFile`, `TdmsGroup`, `TdmsChannel`
  - `new()` constructors for creating empty instances
  - Improved ergonomics for programmatic file creation
- **Flexible String Parameters**: All name parameters now accept `impl Into<String>`
  - Works with `&str`, `String`, and `format!()` results
  - Reduces need for explicit `.to_string()` calls
  - Backward compatible with existing code

### Changed
- **Writer API Returns Results**: All writer methods now return `Result<T, TdmsError>`
  - `add_group()`, `add_channel()`, `add_property()` can fail with validation errors
  - Enables proper error handling and validation
  - Breaking change: existing code needs `?` operators or error handling
- **IndexMap Dependency**: Added `indexmap = "2.0"` for ordered collections
  - Maintains deterministic iteration order
  - Better user experience with predictable output

### Fixed
- **Comprehensive Error Handling**: All error paths now use specific error types
- **Memory Layout**: Ordered collections provide better cache locality
- **API Consistency**: All similar operations now have consistent return types

### Examples
- **API Improvements Demo**: New example showcasing all improvements
  - Input validation demonstration
  - Convenience methods usage
  - Display trait examples
  - Ordered collections verification

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