📘 TDMS Rust API — Exhaustive Design Specification

Status: Design-complete (no implementation)
Audience: Rust library authors, reviewers, FFI authors, LLMs
Scope: Reader + Writer + Safety + Performance + Concurrency
Non-goal: Teaching Rust or TDMS basics

0. Design Philosophy (Read First)

This library is designed under the following hard constraints:

Zero-copy when possible, never unsafe

Lazy by default

Explicit over implicit

Invalid states must be unrepresentable

No UB even on malicious files

Concurrency must be provably safe

Performance contracts must be explicit

Failure modes must be deterministic

If a feature cannot satisfy these constraints, it is explicitly unsupported.

1. High-Level Architecture
tdms
├── reader
│   ├── TdmsFile
│   ├── TdmsGroup
│   ├── TdmsChannel
│   ├── TdmsSlice
│   └── metadata/
│
├── writer
│   ├── TdmsWriter
│   ├── WriterGroup
│   └── WriterChannel<T>
│
├── io
│   ├── mmap.rs
│   └── buffered.rs
│
├── dtype
│   ├── TdmsDType
│   └── validation.rs
│
├── error.rs
└── lib.rs


Separation of concerns is non-negotiable.

2. Core Types Overview
Public API Surface
pub struct TdmsFile;
pub struct TdmsGroup<'f>;
pub struct TdmsChannel<'f>;
pub struct TdmsSlice<'a>;

pub struct TdmsWriter;
pub struct WriterGroup<'w>;
pub struct WriterChannel<'w, T>;

Ownership Model (Critical)

TdmsFile owns:

file descriptor

memory map (if enabled)

metadata index

TdmsGroup<'f> borrows from TdmsFile

TdmsChannel<'f> borrows from TdmsFile

TdmsSlice<'a> borrows from channel data and must never outlive file

This is enforced by lifetimes.

3. Error Model (Mandatory)
Unified Error Type
#[derive(thiserror::Error, Debug)]
pub enum TdmsError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("invalid TDMS format: {0}")]
    InvalidFormat(String),

    #[error("unsupported TDMS feature: {0}")]
    Unsupported(String),

    #[error("compression not supported (found: {format})")]
    CompressionNotSupported { format: String },

    #[error("group not found: {0}")]
    GroupNotFound(String),

    #[error("channel not found: {group}/{channel}")]
    ChannelNotFound { group: String, channel: String },

    #[error("dtype mismatch: expected {expected}, found {found}")]
    DTypeMismatch {
        expected: TdmsDType,
        found: TdmsDType,
    },

    #[error("property too large: {name} ({size} bytes)")]
    PropertyTooLarge { name: String, size: usize },

    #[error("file is closed")]
    FileClosed,

    #[error("out of bounds read")]
    OutOfBounds,
}

Error Rules

Never panic on malformed files

Never return partial success

Errors are fail-fast

Iterators terminate immediately on error

4. DType System
TdmsDType
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TdmsDType {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Bool,
    Utf8,
}

Properties
pub enum PropertyValue {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
}

Property Constraints

Max property size: 1MB

UTF-8 validated

No recursive values

No user mutation after read

5. ChannelData (Zero-Copy Core)
enum ChannelData<'a> {
    /// Backed by mmap — zero-copy
    Mmap(&'a [u8]),

    /// Owned buffer (fallback)
    Owned(Vec<u8>),
}

Selection Rules
Condition	Variant
mmap enabled + contiguous	Mmap
fragmented segments	Owned
endian conversion needed	Owned
compression	error
6. TdmsFile
impl TdmsFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, TdmsError>;

    pub fn groups(&self) -> impl Iterator<Item = TdmsGroup<'_>>;

    pub fn group(&self, name: &str) -> Result<TdmsGroup<'_>, TdmsError>;
}

Metadata Loading Strategy
Data	When Loaded
File index	open()
Group list	open()
Channel list	group()
Properties	on access
Raw data	read()
File Descriptor Rules

Exactly one FD per TdmsFile

FD held until drop

mmap keeps FD alive

7. TdmsChannel
impl<'f> TdmsChannel<'f> {
    pub fn dtype(&self) -> TdmsDType;
    pub fn len(&self) -> usize;

    pub fn read(&self, range: Range<usize>)
        -> Result<TdmsSlice<'_>, TdmsError>;

    pub fn iter_chunks(
        &self,
        chunk_size: usize,
    ) -> ChunkIter<'_>;
}

Segment Handling (Important)

Channels may span multiple segments

read() defragments automatically

Zero-copy only if contiguous

Users never see segments

8. TdmsSlice
pub struct TdmsSlice<'a> {
    data: ChannelData<'a>,
    dtype: TdmsDType,
    len: usize,
}

Typed Access
impl<'a> TdmsSlice<'a> {
    pub fn as_typed<T: Pod>(&self) -> Result<&'a [T], TdmsError>;
}

Validation Rules

size_of::<T>() == dtype.itemsize()

Alignment verified

Endianness verified

Otherwise → DTypeMismatch

Zero-Copy Detection
pub fn is_zero_copy(&self) -> bool;

9. Chunk Iteration Semantics
pub struct ChunkIter<'a>;


Fail-fast

Deterministic ordering

No partial chunks on error

Lifetime tied to file

10. Concurrency Model
Reader
Type	Send	Sync
TdmsFile	✅	✅
TdmsChannel	✅	✅
TdmsSlice	✅	❌

Safe concurrent reads

No shared mutable state

OS handles paging

Writer
Type	Send	Sync
TdmsWriter	✅	❌
11. Writer API
impl TdmsWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, TdmsError>;

    pub fn add_group(&mut self, name: &str)
        -> Result<WriterGroup<'_>, TdmsError>;

    pub fn abort(self) -> Result<(), TdmsError>;
}

WriterGroup
pub struct WriterGroup<'w> {
    _writer: PhantomData<&'w mut TdmsWriter>,
}


Prevents dangling group handles.

12. WriterChannel
impl<'w, T: Pod> WriterChannel<'w, T> {
    pub fn write(&mut self, data: &[T]) -> Result<(), TdmsError>;
}

Write Semantics

Segment-per-write

No implicit buffering

OS handles buffering

Abort deletes file

Close finalizes metadata

13. Unsupported (Explicit)
Feature	Status
Append mode	❌
Compression	❌
Random write	❌
Mutable metadata	❌
In-place modification	❌

Errors are explicit.

14. Safety Guarantees

No unsafe exposed publicly

Internal unsafe audited & minimal

No UB on malformed files

Integer overflow checked

All offsets bounds-checked

15. Fuzzing & Hardening (Required)
#[cfg(fuzzing)]
pub fn fuzz_parse(data: &[u8]) {
    let _ = TdmsFile::from_bytes(data);
}


Must never panic.

16. Rationale Summary
Choice	Why
Range instead of start/count	Rust idiomatic
Lifetimes everywhere	Prevent UAF
No append	Prevent corruption
Fail-fast iterators	Predictable
Explicit zero-copy	No lies
17. Final Statement

This design prioritizes:

Correctness first, performance second, ergonomics third

Because performance without correctness is a bug factory.

This API is:

suitable for scientific computing

safe for untrusted files

scalable to TB-scale datasets

realistic for official crate publication
