📕 TDMS Rust API Design Specification

Status: Design-complete
Audience: Systems programmers, crate maintainers
Scope: Core engine, lifetimes, safety guarantees

1. Design Principles

Zero-copy where provably safe

No UB on malformed files

Lifetimes enforce file validity

Explicit error handling

Concurrency without locks

2. Core Types
pub struct TdmsFile;
pub struct TdmsGroup<'f>;
pub struct TdmsChannel<'f>;

Traits

TdmsFile: Send + Sync

TdmsWriter: Send, not Sync

3. Channel Data Representation
enum ChannelData<'a> {
    Mmap(&'a [u8]),
    Owned(Vec<u8>),
}

Selection Rules

Mmap: contiguous, aligned, no conversion

Owned: fallback (segments, endian, safety)

4. Reading Data
impl TdmsChannel<'_> {
    pub fn read(
        &self,
        range: Range<usize>
    ) -> Result<TdmsSlice<'_>, TdmsError>;
}

Range Semantics

Half-open [start..end)

Idiomatic Rust

Python bindings convert start/count

5. TdmsSlice
pub struct TdmsSlice<'a> {
    data: ChannelData<'a>,
    dtype: TdmsDType,
}

Typed Access
slice.as_typed::<f64>()?;

Validation

size_of::<T>() == dtype.itemsize

Alignment verified

Endianness verified

Otherwise: error (never UB)

6. Chunk Iteration
for chunk in channel.chunks(1_000_000) {
    let slice = chunk?;
    process(slice);
}

Semantics

Iterator yields Result<TdmsSlice>

Fail-fast

Iterator terminates on error

7. Properties
type Properties = HashMap<String, PropertyValue>;

enum PropertyValue {
    String(String),
    F64(f64),
    I32(i32),
}

Constraints

Max size: 1 MB per property

Larger → PropertyTooLarge

8. Metadata & Segments

Metadata indexed on open

Raw data never loaded eagerly

Reads transparently span segments

Zero-copy only if contiguous

9. Writer API
let mut w = TdmsWriter::create("out.tdms")?;
let mut g = w.add_group("DAQ")?;
let mut ch = g.add_channel::<f64>("Voltage")?;
ch.write(&data)?;
w.close()?;

WriterGroup Lifetime
pub struct WriterGroup<'w> {
    _writer: PhantomData<&'w mut TdmsWriter>,
}


Prevents dangling groups.

10. Abort Semantics
w.abort()?; // deletes partial file


Consumes writer

Prevents use-after-abort

11. Concurrency

Multiple readers: safe

mmap: concurrent reads OK

Writer: single-threaded

Writer is Send, not Sync

12. Error Hierarchy
enum TdmsError {
    Io(io::Error),
    DTypeMismatch,
    CompressionNotSupported,
    PropertyTooLarge,
    CorruptFile,
}

13. Compression Handling
Err(TdmsError::CompressionNotSupported)


Detected early

Never silently ignored

14. Platform Support
Platform	Status
Linux	mmap
macOS	mmap
Windows	mmap
WASM	Owned fallback
Embedded	Owned only
15. Safety & Fuzzing

Never panic on malformed input

All parsing returns Result

Recommended: cargo-fuzz

pub fn fuzz_parse(data: &[u8]) {
    let _ = TdmsFile::from_bytes(data);
}

16. Performance Guarantees

O(1) open

O(1) slice creation

Zero-copy where possible

No hidden allocations
