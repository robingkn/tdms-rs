# TDMS-RS API Simplification & Refactoring Analysis

**Date:** 2024  
**Author:** Senior Rust Library Architect Review  
**Scope:** Public API surface, type system, lazy loading, internal architecture, Python bindings readiness

---

## 1. Executive Summary

### Overall API Complexity Assessment

The tdms-rs library presents a **moderately complex** API surface with several areas where simplification would materially improve clarity, learnability, and maintainability. The current design shows signs of incremental evolution rather than intentional simplification, resulting in:

- **Redundant access patterns** (multiple ways to do the same thing)
- **Implicit lazy loading** that obscures I/O behavior
- **Type-specific method explosion** (13+ `as_*` methods)
- **Asymmetric reader/writer APIs**
- **Leaky abstractions** exposing internal TDMS format details

### Breaking Changes Justification

**Recommendation: Minor cleanup now, v2 redesign justified**

The current API is functional and correct, but several breaking changes would significantly reduce cognitive load:

1. **Channel data access API** - The proliferation of type-specific accessors creates unnecessary API surface area
2. **Lazy loading semantics** - Implicit I/O via `OnceLock` makes it unclear when disk reads occur
3. **Reader/Writer asymmetry** - Different patterns for similar operations increase learning curve
4. **Property access patterns** - Multiple overlapping ways to access properties with inconsistent type coercion

A **v2 redesign** would be justified if prioritizing:
- Long-term maintainability over backward compatibility
- Python bindings simplicity
- Clear separation of metadata vs. data operations
- Explicit I/O control

### High-Level Recommendation

**Option A (Conservative):** Minor cleanup without breaking changes
- Consolidate property accessors
- Add explicit `load()` methods alongside implicit lazy loading
- Document lazy loading behavior clearly
- **Impact:** Low risk, incremental improvement

**Option B (Moderate):** Targeted breaking changes in v1.x
- Simplify channel data access API
- Make lazy loading explicit
- Unify reader/writer patterns
- **Impact:** Medium risk, significant clarity gains

**Option C (Aggressive):** Full v2 redesign
- Complete API reset with lessons learned
- Separate metadata-only and data-backed types
- Explicit I/O throughout
- **Impact:** High risk, maximum clarity

**Recommended:** **Option B** - The complexity issues are real and worth fixing, but a full v2 reset may be premature. Targeted breaking changes can address the worst pain points while maintaining most of the API.

---

## 2. Simplification Opportunities (Prioritized)

### [Priority P0] Channel Data Access API Explosion

**Area:** `TdmsChannel` data accessors

**Current Complexity:**
- 13 type-specific accessor methods: `as_f64()`, `as_f32()`, `as_i32()`, `as_i8()`, `as_i16()`, `as_i64()`, `as_u8()`, `as_u16()`, `as_u32()`, `as_u64()`, `as_bool()`, `as_string()`, `as_timestamps()`
- Additional convenience methods: `as_numeric()`, `as_timestamps_f64()`, `timestamps_to_unix()`
- Direct access via `channel.data: Option<TdmsData>` with pattern matching
- All methods implicitly trigger lazy loading via `ensure_data_loaded()`

**Why It's Complex Today:**
1. **API surface bloat:** 13+ methods for what is essentially one operation: "get the data"
2. **Type discovery overhead:** Users must know the exact Rust type to call the right method
3. **Silent failures:** Methods return `Option<&[T]>` - unclear if data is missing or wrong type
4. **Inconsistent patterns:** Some methods return slices (`&[f64]`), others return owned `Vec<f64>`
5. **Dual access paths:** Both `as_f64()` and `channel.data` exist, creating confusion about which to use

**Simpler Conceptual Model:**
```rust
// Single unified accessor with type inference
impl TdmsChannel {
    // Primary accessor - returns the enum directly
    pub fn data(&self) -> Result<&TdmsData>;
    
    // Type-safe extraction (only if you need specific type)
    pub fn try_as<T: FromTdmsData>(&self) -> Option<&T>;
}

// Or even simpler: just expose data() and let users match
// Remove all as_* methods entirely
```

**Breaking Change Required:** Yes

**Why It's Worth It:**
- Reduces API surface from 13+ methods to 1-2 methods
- Forces explicit type handling via pattern matching (more Rust-idiomatic)
- Makes type mismatches obvious at compile time
- Easier to document and understand
- Python bindings become simpler (one method instead of 13)

**Impact:** High - Affects all users, but migration is straightforward (replace `as_f64()` with `data()?` and match)

**Migration Strategy:**
- Deprecate all `as_*` methods in v1.1
- Provide `data()` as primary accessor
- Keep deprecated methods for 2-3 versions
- Provide migration guide with examples

---

### [Priority P0] Implicit Lazy Loading via OnceLock

**Area:** Lazy data loading semantics

**Current Complexity:**
- Data loading happens implicitly when calling any `as_*` method
- Uses `OnceLock` internally - users don't know when I/O occurs
- `ensure_data_loaded()` exists but is "called automatically"
- Three-state system: `data: Option<TdmsData>`, `cache: OnceLock<TdmsData>`, `data_locations: Vec<DataLocation>`
- `data_len()` works without loading data (uses `data_locations`), but other operations trigger I/O

**Why It's Complex Today:**
1. **Hidden I/O:** Users can't tell when expensive disk reads happen
2. **Error handling confusion:** `as_f64()` returns `Option` but I/O errors are swallowed
3. **State ambiguity:** Three different places data can live (`data`, `cache`, or on disk)
4. **Testing difficulty:** Hard to mock or control I/O behavior
5. **Python bindings problem:** Implicit I/O doesn't map well to Python's explicit model

**Simpler Conceptual Model:**

**Option 1: Explicit Loading**
```rust
// Separate metadata-only and data-backed types
pub struct TdmsChannelMetadata {
    pub properties: IndexMap<String, PropertyValue>,
    pub data_type: Option<DataType>,
    pub data_len: usize,
    // ... no data access
}

pub struct TdmsChannel {
    metadata: TdmsChannelMetadata,
    data: TdmsData,  // Always loaded
}

impl TdmsChannelMetadata {
    pub fn load_data(&self) -> Result<TdmsChannel>;
}
```

**Option 2: Explicit Load Method**
```rust
impl TdmsChannel {
    // Metadata always available
    pub fn data_len(&self) -> usize;
    pub fn data_type(&self) -> Option<DataType>;
    
    // Explicit data loading
    pub fn load(&mut self) -> Result<()>;
    pub fn data(&self) -> Option<&TdmsData>;  // None until load() called
}
```

**Breaking Change Required:** Yes (for Option 1), Maybe (for Option 2)

**Why It's Worth It:**
- Makes I/O explicit and predictable
- Better error handling (errors visible at load time)
- Easier to test and mock
- Python bindings become natural: `channel.load()` then `channel.data`
- Users can choose when to pay I/O cost

**Impact:** High - Changes fundamental usage pattern, but improves clarity significantly

**Migration Strategy:**
- Add explicit `load()` method alongside implicit loading
- Deprecate implicit loading in v1.2
- Provide both patterns during transition
- Update all examples to use explicit loading

---

### [Priority P1] Reader/Writer API Asymmetry

**Area:** `TdmsFile` (reader) vs `TdmsFileWriter` (writer)

**Current Complexity:**
- **Reader:** `TdmsFile::load()` returns `TdmsFile` with `groups: IndexMap<String, TdmsGroup>`
- **Writer:** `TdmsFileWriter::new()` returns writer, `add_group()` returns `&mut TdmsGroupWriter`
- Different patterns for accessing channels:
  - Reader: `file.get_channel("group", "channel")` or `file.group("group")?.channel("channel")`
  - Writer: `writer.add_group("group")?.add_channel("name", data)?`
- Reader uses `IndexMap`, Writer uses `BTreeMap` internally (but exposes `IndexMap`-like API)
- Reader has lazy loading, Writer requires all data upfront

**Why It's Complex Today:**
1. **Different mental models:** Reader is "load then access", Writer is "build then write"
2. **Inconsistent return types:** Reader returns owned structs, Writer returns mutable references
3. **Different collection types:** `IndexMap` vs `BTreeMap` - why?
4. **Path access inconsistency:** Reader has `get_channel(group, channel)`, Writer uses builder pattern

**Simpler Conceptual Model:**
```rust
// Unified builder pattern for both
pub struct TdmsFileBuilder {
    // ... internal state
}

impl TdmsFileBuilder {
    pub fn add_group(&mut self, name: &str) -> &mut TdmsGroupBuilder;
    pub fn build(self) -> TdmsFile;  // For reading
    pub fn write(self, path: &Path) -> Result<()>;  // For writing
}

// Or: Make writer use same structure as reader
pub struct TdmsFileWriter {
    pub groups: IndexMap<String, TdmsGroupWriter>,  // Same as reader
    pub properties: IndexMap<String, PropertyValue>,
}

impl TdmsFileWriter {
    pub fn write(&self, path: &Path) -> Result<()>;
}
```

**Breaking Change Required:** Yes

**Why It's Worth It:**
- Single mental model for reading and writing
- Consistent API patterns reduce learning curve
- Easier to convert between reader and writer structures
- Python bindings: same object model for both operations

**Impact:** Medium - Affects writer API primarily, reader API mostly unchanged

**Migration Strategy:**
- Introduce new unified API alongside old API
- Deprecate `TdmsFileWriter` builder pattern
- Provide migration examples
- Keep old API for 2 versions

---

### [Priority P1] Property Access Pattern Proliferation

**Area:** Property access methods

**Current Complexity:**
- Direct access: `channel.properties.get("name")` returns `Option<PropertyValue>`
- Type-specific getters: `get_string_property()`, `get_double_property()`, `get_i32_property()`, `get_i64_property()`
- Convenience methods: `unit()`, `increment()`, `start_time()`, `sample_count()`, `description()`, `sensor_type()`
- Type coercion: `get_double_property()` accepts both `Double` and `Float` variants
- Inconsistent return types: Some return `Option<&str>`, others `Option<f64>`, others `Option<i32>`

**Why It's Complex Today:**
1. **Too many ways to do the same thing:** 4+ ways to access a property
2. **Magic string dependencies:** Convenience methods hardcode property names
3. **Type coercion inconsistency:** Some methods coerce types, others don't
4. **Unclear precedence:** When to use which method?

**Simpler Conceptual Model:**
```rust
impl TdmsChannel {
    // Single property accessor with type parameter
    pub fn property<T: From<PropertyValue>>(&self, name: &str) -> Option<T>;
    
    // Or: Just use direct access and let users match
    // Remove all convenience methods, keep only direct access
}

// Remove: get_string_property, get_double_property, unit(), increment(), etc.
// Keep: Direct access via properties.get()
```

**Breaking Change Required:** Yes (removing convenience methods)

**Why It's Worth It:**
- Reduces API surface significantly
- Forces explicit property name handling (no magic strings)
- More flexible (users can define their own helpers)
- Python bindings: single `property(name)` method

**Impact:** Low-Medium - Many users likely use convenience methods, but migration is straightforward

**Migration Strategy:**
- Deprecate convenience methods
- Provide `property<T>()` generic method
- Show examples of using direct access
- Keep deprecated methods for 2 versions

---

### [Priority P2] TdmsData Enum Explosion

**Area:** `TdmsData` enum with 13 variants

**Current Complexity:**
- Enum with variants for every integer size: `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`
- Plus: `Float`, `Double`, `String`, `Boolean`, `TimeStamp`
- Every variant wraps `Vec<T>`
- Pattern matching required everywhere
- `extend()` method has 13 match arms

**Why It's Complex Today:**
1. **Verbose pattern matching:** Users must handle 13 cases
2. **Type erasure:** Can't use generic code across numeric types
3. **Maintenance burden:** Adding new types requires updating many match statements

**Simpler Conceptual Model:**

**Option 1: Type Erasure with Trait**
```rust
pub trait TdmsDataType: Clone {
    fn read_from<R: Read>(reader: &mut R, count: usize) -> Result<Self>;
    // ...
}

pub enum TdmsData {
    Numeric(Box<dyn NumericData>),  // Trait object for all numeric types
    String(Vec<String>),
    Boolean(Vec<bool>),
    TimeStamp(Vec<(i64, u64)>),
}
```

**Option 2: Keep Current Design** (Recommended)
- Current enum is actually fine - explicit types are valuable
- Problem is accessor explosion, not enum itself
- Keep enum, simplify accessors (see P0)

**Breaking Change Required:** No (if keeping current design)

**Why Current Design is Actually Good:**
- Type safety: Can't accidentally mix `i32` and `i64` data
- Zero-cost abstractions: No trait object overhead
- Clear and explicit: Users know exactly what types they have
- Pattern matching is Rust-idiomatic

**Impact:** Low - Current design is fine, focus on accessor simplification instead

---

### [Priority P2] Internal Details Leaking to Public API

**Area:** `pub(crate)` fields and internal types

**Current Complexity:**
- `TdmsChannel` has `pub(crate)` fields: `data_locations`, `file_path`, `cache`, `data_type`
- `TdmsFile` has `pub(crate)` field: `_file_path`
- `DataLocation`, `RawDataMeta` are internal but might be needed for advanced use cases
- Users might need to understand TDMS segment structure

**Why It's Complex Today:**
1. **Leaky abstraction:** Internal TDMS format details visible
2. **Unclear boundaries:** What's public API vs. internal?
3. **Documentation burden:** Must document internal fields

**Simpler Conceptual Model:**
```rust
// Make all internal fields private
pub struct TdmsChannel {
    pub properties: IndexMap<String, PropertyValue>,
    // All other fields private
    // Provide public methods for everything users need
}

// If advanced users need internal details, provide explicit API
impl TdmsChannel {
    pub fn segment_info(&self) -> SegmentInfo;  // Explicit API for advanced use
}
```

**Breaking Change Required:** No (just make fields private)

**Why It's Worth It:**
- Cleaner abstraction boundaries
- Can change internal representation without breaking users
- Forces explicit API design

**Impact:** Low - Mostly documentation/clarity improvement

---

### [Priority P2] Error Type Design

**Area:** `TdmsError` enum

**Current Complexity:**
- Uses `thiserror` for error generation
- Mix of I/O errors, format errors, and domain errors
- Some errors include context strings (`GroupNotFound(String)`)
- `NotImplemented(String)` used for unsupported features

**Why It's Complex Today:**
1. **String-based errors:** `NotImplemented(String)` requires string formatting
2. **Mixed error categories:** I/O, format, domain errors all in one enum
3. **Unclear error recovery:** Hard to know which errors are recoverable

**Simpler Conceptual Model:**
```rust
#[derive(Error, Debug)]
pub enum TdmsError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid TDMS format")]
    InvalidFormat { details: String, offset: Option<u64> },
    
    #[error("Object not found: {path}")]
    NotFound { path: String },
    
    // Remove NotImplemented - use proper error types
}
```

**Breaking Change Required:** Maybe (if removing `NotImplemented`)

**Why It's Worth It:**
- Better error messages with context
- Easier error handling for users
- More professional API

**Impact:** Low - Mostly internal improvement

---

## 3. "If We Did a v2" Section

Assume we're allowed to break everything and optimize purely for clarity. Here's what a v2 API would look like:

### Core Design Principles

1. **Explicit over Implicit:** All I/O operations are explicit
2. **Metadata First:** Separate metadata-only and data-backed types
3. **Single Responsibility:** Each type has one clear purpose
4. **Consistent Patterns:** Reader and writer use same structures
5. **Type Safety:** Leverage Rust's type system, don't fight it

### What We Would Remove

1. **All `as_*` methods** - Replace with single `data()` method and pattern matching
2. **Implicit lazy loading** - Make all data loading explicit via `load()` method
3. **Convenience property methods** - `unit()`, `increment()`, etc. - Use direct property access
4. **`TdmsFileWriter` builder pattern** - Use same structure as reader
5. **`NotImplemented` error variant** - Use proper error types
6. **`pub(crate)` fields** - Make everything private, provide explicit APIs
7. **Type coercion in property accessors** - Be explicit about types

### What We Would Merge

1. **Reader and Writer structures** - Same `TdmsFile` structure for both
2. **Property access patterns** - Single `property<T>()` method instead of multiple getters
3. **Channel access methods** - Single `channel(path)` method instead of `get_channel(group, channel)`

### What We Would Make Explicit

1. **Data loading** - `load()` method required before accessing data
2. **I/O operations** - All file operations return `Result` and are explicit
3. **Type conversions** - No implicit coercion, users must be explicit
4. **Error handling** - All errors are typed and recoverable where possible

### What We Would Hide Entirely

1. **TDMS segment structure** - Users shouldn't need to know about segments
2. **Data locations and offsets** - Internal implementation detail
3. **OnceLock caching** - Implementation detail, not API surface
4. **Raw data metadata** - Internal parsing detail

### Proposed v2 API Structure

```rust
// ============================================================================
// Core Types - Metadata First
// ============================================================================

/// Metadata-only file structure (no data loaded)
pub struct TdmsFileMetadata {
    pub properties: IndexMap<String, PropertyValue>,
    pub groups: IndexMap<String, TdmsGroupMetadata>,
}

/// Metadata-only group structure
pub struct TdmsGroupMetadata {
    pub properties: IndexMap<String, PropertyValue>,
    pub channels: IndexMap<String, TdmsChannelMetadata>,
}

/// Metadata-only channel structure
pub struct TdmsChannelMetadata {
    pub properties: IndexMap<String, PropertyValue>,
    pub data_type: Option<DataType>,
    pub data_len: usize,
}

// ============================================================================
// Data-Backed Types
// ============================================================================

/// File with all data loaded
pub struct TdmsFile {
    metadata: TdmsFileMetadata,
    // Data is loaded into channels
}

/// Channel with data loaded
pub struct TdmsChannel {
    metadata: TdmsChannelMetadata,
    data: TdmsData,  // Always Some after load()
}

// ============================================================================
// Loading API
// ============================================================================

impl TdmsFileMetadata {
    /// Load metadata only (fast, no I/O for data)
    pub fn load_metadata(path: &Path) -> Result<Self>;
    
    /// Load all data (slow, full I/O)
    pub fn load(path: &Path) -> Result<TdmsFile>;
}

impl TdmsChannelMetadata {
    /// Load data for this channel
    pub fn load_data(&self, file_path: &Path) -> Result<TdmsChannel>;
}

// ============================================================================
// Data Access API
// ============================================================================

impl TdmsChannel {
    /// Get the data enum - users pattern match
    pub fn data(&self) -> &TdmsData;
    
    /// Get metadata
    pub fn metadata(&self) -> &TdmsChannelMetadata;
}

// ============================================================================
// Property Access API
// ============================================================================

impl TdmsChannelMetadata {
    /// Get property with type conversion
    pub fn property<T: TryFrom<PropertyValue>>(&self, name: &str) -> Option<T>;
    
    /// Direct property access
    pub fn properties(&self) -> &IndexMap<String, PropertyValue>;
}

// ============================================================================
// Writing API (Same Structure!)
// ============================================================================

/// Writer uses same structure as reader
pub struct TdmsFileWriter {
    pub properties: IndexMap<String, PropertyValue>,
    pub groups: IndexMap<String, TdmsGroupWriter>,
}

pub struct TdmsGroupWriter {
    pub properties: IndexMap<String, PropertyValue>,
    pub channels: IndexMap<String, TdmsChannelWriter>,
}

pub struct TdmsChannelWriter {
    pub properties: IndexMap<String, PropertyValue>,
    pub data: TdmsData,
}

impl TdmsFileWriter {
    pub fn write(&self, path: &Path) -> Result<()>;
}

// ============================================================================
// Channel Access API (Unified)
// ============================================================================

impl TdmsFile {
    /// Access channel by path: "/'group'/'channel'" or just "group/channel"
    pub fn channel(&self, path: &str) -> Option<&TdmsChannel>;
    
    /// Iterate channels
    pub fn channels(&self) -> impl Iterator<Item = (&str, &str, &TdmsChannel)>;
}
```

### Key v2 Improvements

1. **Metadata/Data Separation:** Clear distinction between metadata (always available) and data (requires load)
2. **Explicit Loading:** No hidden I/O, users control when data is loaded
3. **Unified Structures:** Reader and writer use same types
4. **Simplified Access:** Single `data()` method, pattern matching for types
5. **Clean Property API:** Single `property<T>()` method with type conversion
6. **Path-Based Access:** Unified channel access via path strings
7. **No Leaky Abstractions:** All internal details hidden

### Python Bindings Readiness

The v2 API maps cleanly to Python:

```python
# Python equivalent
file = TdmsFile.load_metadata("data.tdms")  # Fast, metadata only
channel_meta = file.groups["Sensors"].channels["Temperature"]

# Explicit loading
channel = channel_meta.load_data("data.tdms")

# Simple data access
data = channel.data()  # Returns Python list/array based on type
if isinstance(data, list) and data and isinstance(data[0], float):
    # Handle float data
    pass

# Property access
unit = channel.metadata.property("wf_unit_string", str)
```

---

## 4. Things That Should NOT Change

### ✅ Keep: TdmsData Enum Design

**Why:** The enum with explicit variants is actually excellent design:
- Type safety: Can't accidentally mix `i32` and `i64`
- Zero-cost: No trait object overhead
- Explicit: Users know exactly what types they have
- Pattern matching is idiomatic Rust

**Don't change:** Keep the enum, just simplify accessors (see P0)

### ✅ Keep: IndexMap for Groups/Channels

**Why:** Preserves insertion order, which matters for TDMS files:
- Deterministic iteration
- Matches TDMS file structure
- Good performance characteristics

**Don't change:** Keep `IndexMap` for all collections

### ✅ Keep: PropertyValue Enum

**Why:** Clean representation of TDMS property types:
- Type-safe property values
- Easy to extend
- Good for serialization

**Don't change:** Current design is solid

### ✅ Keep: Error Handling with thiserror

**Why:** `thiserror` provides excellent error handling:
- Good error messages
- Easy to extend
- Standard Rust pattern

**Don't change:** Keep error structure, just improve error variants (see P2)

### ✅ Keep: Path-Based Channel Access

**Why:** `get_channel(group, channel)` is clear and ergonomic:
- Matches TDMS mental model
- Easy to understand
- Good for common cases

**Don't change:** Keep this pattern, maybe add path-based access as alternative

### ✅ Keep: Builder Pattern for Writing

**Why:** Builder pattern is good for constructing complex structures:
- Clear construction flow
- Type-safe
- Easy to use

**Don't change:** Keep builder pattern, just make structures match reader

---

## 5. Risk & Migration Notes

### High-Risk Breaking Changes

1. **Removing `as_*` methods (P0)**
   - **Risk:** High - Affects all users
   - **Mitigation:** 
     - Deprecate in v1.1, remove in v2.0
     - Provide clear migration guide
     - Add `data()` method immediately
     - Keep deprecated methods for 2-3 versions
   - **Migration effort:** Low per-user (find/replace `as_f64()` → `data()?` + match)

2. **Making lazy loading explicit (P0)**
   - **Risk:** High - Changes fundamental usage pattern
   - **Mitigation:**
     - Add explicit `load()` method alongside implicit
     - Deprecate implicit loading gradually
     - Provide both patterns during transition
     - Update all examples
   - **Migration effort:** Medium per-user (add `load()` calls, handle errors)

### Medium-Risk Breaking Changes

3. **Unifying reader/writer structures (P1)**
   - **Risk:** Medium - Affects writer API primarily
   - **Mitigation:**
     - Introduce new API alongside old
     - Provide migration examples
     - Keep old API for 2 versions
   - **Migration effort:** Medium per-user (restructure writer code)

4. **Removing convenience property methods (P1)**
   - **Risk:** Medium - Many users likely use these
   - **Mitigation:**
     - Deprecate gradually
     - Provide `property<T>()` method
     - Show examples of direct access
   - **Migration effort:** Low per-user (replace method calls)

### Low-Risk Changes

5. **Making internal fields private (P2)**
   - **Risk:** Low - Mostly affects advanced users
   - **Mitigation:**
     - Provide explicit APIs for advanced use cases
     - Document migration path
   - **Migration effort:** Low (if any)

6. **Improving error types (P2)**
   - **Risk:** Low - Mostly internal
   - **Mitigation:**
     - Add new error variants
     - Keep old variants deprecated
   - **Migration effort:** Very low

### Migration Strategy Recommendations

1. **Versioned Approach:**
   - v1.1: Add new APIs, deprecate old ones
   - v1.2: Keep deprecated APIs, add migration warnings
   - v2.0: Remove deprecated APIs, breaking changes

2. **Gradual Migration:**
   - Provide both old and new APIs simultaneously
   - Document preferred patterns
   - Update examples to use new APIs
   - Let users migrate at their own pace

3. **Migration Tools:**
   - Provide `cargo fix` rules where possible
   - Create migration guide with before/after examples
   - Provide compatibility shims for common patterns

4. **Communication:**
   - Clear deprecation messages pointing to new APIs
   - Blog post explaining rationale
   - Migration guide with examples
   - Community feedback period before v2

### Python Bindings Considerations

Many of these simplifications make Python bindings **dramatically simpler**:

1. **Explicit loading:** Maps naturally to Python's explicit model
2. **Single `data()` method:** One method instead of 13
3. **Unified structures:** Same object model for reading and writing
4. **Property API:** Single `property()` method maps cleanly

**Recommendation:** If Python bindings are a priority, these breaking changes are even more justified.

---

## Conclusion

The tdms-rs API is functional but has accumulated complexity that would benefit from simplification. The highest-impact changes are:

1. **Simplifying channel data access** (P0) - Removes 13+ methods, improves clarity
2. **Making lazy loading explicit** (P0) - Improves I/O predictability and Python bindings
3. **Unifying reader/writer APIs** (P1) - Reduces learning curve
4. **Consolidating property access** (P1) - Reduces API surface

A **v2 redesign** would be justified if prioritizing long-term maintainability and Python bindings simplicity. However, **targeted breaking changes in v1.x** can address the worst pain points without a full reset.

**Recommended Path Forward:**
1. Implement P0 changes (data access + explicit loading) in v1.1 with deprecations
2. Implement P1 changes (unified APIs + property consolidation) in v1.2
3. Remove deprecated APIs in v2.0
4. Consider full v2 redesign if Python bindings become critical

The current API is **good enough** for v1.0, but these simplifications would make it **excellent** for long-term maintenance and Python bindings.

