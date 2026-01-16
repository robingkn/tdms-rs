📕 TDMS API DESIGN — RUST

Audience: Systems programmers, DAQ backends
Goals: Zero UB, zero-copy, predictable performance
Philosophy: Make invalid states unrepresentable

1. Core Types
pub struct TdmsFile { /* immutable */ }
pub struct TdmsGroup<'a> { /* view */ }
pub struct TdmsChannel<'a> { /* view */ }

2. DType System
pub enum TdmsDType {
    I8, U8, I16, U16,
    I32, U32, I64, U64,
    F32, F64,
    Bool,
    String,
}

Introspection
impl TdmsDType {
    pub fn itemsize(&self) -> usize;
    pub fn is_numeric(&self) -> bool;
    pub fn endian(&self) -> Endian;
}

3. Typed Zero-Copy Reads
pub struct TdmsSlice<'a> {
    dtype: TdmsDType,
    bytes: &'a [u8],
}

impl<'a> TdmsSlice<'a> {
    pub fn as_typed<T: Pod>(&self) -> Result<&'a [T], TdmsError>;
}

Safety Checks

size_of::<T>() == dtype.itemsize()

Alignment validated

Endianness validated

Else → Err(TdmsError::TypeMismatch)

4. Properties
pub enum PropertyValue {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Timestamp(DateTime<Utc>),
}

Parsing

Defined by TDMS spec

Explicit from_bytes()

Max size: 1MB

5. Reading API
impl TdmsFile {
    pub fn open(path: impl AsRef<Path>) -> Result<Self>;
    pub fn group(&self, name: &str) -> Option<TdmsGroup>;
}

impl<'a> TdmsChannel<'a> {
    pub fn read(&self) -> Result<TdmsSlice<'a>>;
    pub fn read_range(&self, range: Range<usize>) -> Result<TdmsSlice<'a>>;
    pub fn iter_chunks(&self, size: usize)
        -> impl Iterator<Item = TdmsSlice<'a>>;
}

6. Timestamps (Iterator)
impl<'a> TdmsChannel<'a> {
    pub fn timestamps(&self)
        -> Option<impl Iterator<Item = f64> + 'a>;
}

7. Writer API
pub struct TdmsWriter {
    path: PathBuf,
    file: File,
}

impl TdmsWriter {
    pub fn create(path: impl AsRef<Path>) -> Result<Self>;
    pub fn add_group(&mut self, name: &str) -> TdmsGroupWriter;
    pub fn abort(self) -> io::Result<()>;
}

Concurrency

TdmsWriter: Send

!Sync

Safe to move between threads

Not usable concurrently

8. Error Model
#[derive(thiserror::Error)]
pub enum TdmsError {
    #[error("compression not supported ({format})")]
    CompressionNotSupported { format: String },

    #[error("property too large ({size} bytes)")]
    PropertyTooLarge { size: usize },

    #[error("dtype mismatch")]
    TypeMismatch,

    #[error("file closed")]
    Closed,
}

9. Concurrency Guarantees
Scenario	Allowed
Arc<TdmsFile> multi-thread	✅
mmap + fork	✅
Writer sharing	❌
10. Unsupported (Explicit)

Append mode

Compression

Mutation after write

Auto endian conversion

11. Rust Workflow Examples
Parallel Analysis
let file = Arc::new(TdmsFile::open("data.tdms")?);

(0..8).into_par_iter().for_each(|i| {
    let ch = file.group("G").unwrap().channel("C").unwrap();
    let slice = ch.read_range(i*1_000..(i+1)*1_000).unwrap();
    process(slice.as_typed::<f64>().unwrap());
});

Streaming Write
let mut w = TdmsWriter::create("live.tdms")?;
let g = w.add_group("DAQ");
let mut ch = g.add_channel::<f64>("V");

loop {
    let data = acquire();
    ch.write(&data)?;
}

12. Final Notes

This spec is:

Minimal

Explicit

Safe

Scalable

Backend-agnostic

You can now:

Implement Rust core

Bind Python on top

Publish crate + PyPI package

Generate conformance tests mechanically
