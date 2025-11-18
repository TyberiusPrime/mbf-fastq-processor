# rapidgzip-wrapper Structure

This document describes the structure of the rapidgzip-wrapper subcrate.

## Directory Layout

```
rapidgzip-wrapper/
├── Cargo.toml              # Crate configuration and dependencies
├── README.md               # Main documentation
├── INTEGRATION_TODO.md     # Step-by-step integration guide
├── STRUCTURE.md           # This file
├── .gitignore             # Git ignore rules
├── build.rs               # Build script for compiling C++ code
│
├── src/                   # Rust source code
│   ├── lib.rs            # Main library with safe Rust API
│   └── ffi.rs            # Raw FFI bindings to C wrapper
│
├── cpp/                   # C++ wrapper code
│   ├── rapidgzip_c_wrapper.hpp  # C-style API header
│   └── rapidgzip_c_wrapper.cpp  # C-style API implementation
│
├── examples/              # Usage examples
│   ├── basic_usage.rs    # Simple decompression example
│   ├── seek_example.rs   # Demonstrating seek capability
│   └── benchmark.rs      # Performance comparison
│
└── vendor/               # Third-party dependencies (to be added)
    └── indexed_bzip2/    # rapidgzip source (not yet present)
        └── src/
            └── rapidgzip/
```

## Component Descriptions

### Core Components

#### `src/lib.rs`
The main Rust API providing:
- `ParallelGzipReader` struct - Safe wrapper around C++ library
- `Read` trait implementation - Standard Rust I/O
- `Seek` trait implementation - Random access to decompressed data
- Error handling using `anyhow::Result`
- Safe resource management via RAII (Drop trait)

Key types:
```rust
pub struct ParallelGzipReader { ... }

impl ParallelGzipReader {
    pub fn open<P: AsRef<Path>>(path: P, num_threads: usize) -> Result<Self>
    pub fn from_fd(fd: RawFd, num_threads: usize) -> Result<Self>
    pub fn tell(&self) -> Result<u64>
    pub fn is_eof(&self) -> Result<bool>
    pub fn set_crc32_enabled(&mut self, enabled: bool) -> Result<()>
    pub fn size(&self) -> Result<Option<u64>>
}

impl Read for ParallelGzipReader { ... }
impl Seek for ParallelGzipReader { ... }
impl Drop for ParallelGzipReader { ... }
```

#### `src/ffi.rs`
Low-level FFI bindings:
- Raw C function declarations (`extern "C"`)
- C-compatible type definitions
- No safety guarantees - internal use only
- Maps Rust types to C types

#### `cpp/rapidgzip_c_wrapper.hpp`
C-style API header exposing:
- Opaque `RapidGzipReader` handle type
- C-compatible error codes
- Function declarations for all operations
- No C++ features in the interface

Functions:
- `rapidgzip_open()` - Open file by path
- `rapidgzip_open_fd()` - Open file by descriptor
- `rapidgzip_read()` - Read decompressed data
- `rapidgzip_seek()` - Seek to position
- `rapidgzip_tell()` - Get current position
- `rapidgzip_eof()` - Check end-of-file
- `rapidgzip_set_crc32_enabled()` - Toggle CRC verification
- `rapidgzip_size()` - Get decompressed size
- `rapidgzip_close()` - Free resources

#### `cpp/rapidgzip_c_wrapper.cpp`
C++ implementation that:
- Wraps the rapidgzip C++ template library
- Provides C-compatible interface
- Handles exceptions and converts to error codes
- Manages C++ object lifetimes

**Current Status**: Stubbed - needs actual rapidgzip integration

### Build System

#### `build.rs`
Cargo build script that:
- Uses `cc` crate to compile C++ code
- Sets C++17 standard (required by rapidgzip)
- Configures include paths
- Links C++ standard library (platform-specific)
- Will link zlib and optionally ISA-L when integrated

Build process:
1. Compiles `cpp/rapidgzip_c_wrapper.cpp`
2. Creates static library `librapidgzip_wrapper.a`
3. Links with Rust code
4. Links C++ std library and dependencies

#### `Cargo.toml`
Dependencies:
- **Runtime**: `anyhow`, `libc`
- **Build**: `cc` (C++ compiler), `cmake` (for future use)

### Examples

#### `examples/basic_usage.rs`
Demonstrates:
- Opening a gzip file
- Querying decompressed size
- Reading decompressed data
- Writing to stdout

#### `examples/seek_example.rs`
Demonstrates:
- Random access to decompressed data
- Seeking from start, current position, and end
- Reading from different positions
- Efficient data sampling

#### `examples/benchmark.rs`
Demonstrates:
- Performance testing with different thread counts
- Throughput measurement
- Speedup calculations
- Optimal thread count determination

## Data Flow

```
User Code (Rust)
    ↓
src/lib.rs (Safe Rust API)
    ↓
src/ffi.rs (Raw FFI)
    ↓
cpp/rapidgzip_c_wrapper.cpp (C bridge)
    ↓
vendor/indexed_bzip2/src/rapidgzip/ (C++ library)
    ↓
System libraries (zlib, ISA-L)
```

## Integration Status

### ✅ Complete
- Directory structure
- C bridge header and implementation skeleton
- Rust FFI bindings
- Safe Rust API with standard traits
- Build system configuration
- Documentation
- Examples

### ❌ Incomplete (See INTEGRATION_TODO.md)
- Actual rapidgzip source code
- C++ wrapper implementation (currently stubbed)
- System dependency handling
- Tests
- Benchmarks

## Next Steps

1. Follow `INTEGRATION_TODO.md` to complete integration
2. Add rapidgzip source to `vendor/`
3. Complete C++ wrapper implementation
4. Build and test
5. Add comprehensive tests
6. Create benchmarks
7. Document performance characteristics

## Testing Strategy

Once integrated, tests should cover:

1. **Unit Tests** (in `src/lib.rs`)
   - Open/close operations
   - Read operations
   - Seek operations
   - Error handling
   - Edge cases (empty files, corrupted data)

2. **Integration Tests** (in `tests/`)
   - End-to-end decompression
   - Comparison with standard gzip
   - Large file handling
   - Concurrent readers

3. **Benchmarks** (in `benches/`)
   - Throughput vs thread count
   - Comparison with other gzip implementations
   - Memory usage
   - Seek performance

## Performance Considerations

Expected characteristics once integrated:

- **Throughput**: 20-30x faster than single-threaded gzip on multi-core systems
- **Memory**: Higher than single-threaded (buffers for parallel processing)
- **Latency**: Small overhead for thread coordination
- **Seek**: Much faster than re-decompressing from start
- **Scalability**: Near-linear speedup with available cores

## Dependencies

### Build-time
- C++17 compatible compiler (GCC 7+, Clang 5+)
- `cc` crate for C++ compilation
- zlib development headers

### Runtime
- zlib (dynamic linking, or can be statically linked)
- ISA-L (optional, for improved performance)
- C++ standard library

### Optional
- rpmalloc (for better memory allocation performance)
- NASM (for building ISA-L)
