# rapidgzip-wrapper

Rust wrapper for [librapidgzip](https://github.com/mxmlnkn/rapidgzip), a parallel gzip decompression library.

## Overview

This crate provides safe Rust bindings to the librapidgzip C++ library, which enables fast parallel decompression of gzip files. The library uses multiple threads to decompress gzip files significantly faster than standard single-threaded implementations.

## Architecture

The wrapper consists of three layers:

1. **C++ Library**: The actual rapidgzip library (header-only C++17)
2. **C Bridge** (`cpp/rapidgzip_c_wrapper.*`): A C-style API wrapper around the C++ library
3. **Rust API** (`src/lib.rs`): Safe Rust interface using the C bridge via FFI

## Current Status

⚠️ **INCOMPLETE**: This is a skeleton implementation. The actual rapidgzip source code needs to be integrated.

### What's Done

- ✅ C bridge header and implementation structure
- ✅ Rust FFI bindings
- ✅ Safe Rust API with `Read` and `Seek` traits
- ✅ Build system configuration
- ✅ Error handling infrastructure

### What's Needed

- ❌ Integrate rapidgzip source code into `vendor/`
- ❌ Update build.rs to compile rapidgzip dependencies
- ❌ Complete the C++ wrapper implementation (currently stubbed)
- ❌ Add tests
- ❌ Handle platform-specific dependencies (zlib, ISA-L)

## Integration Steps

### 1. Add rapidgzip Source

The rapidgzip library is part of the [indexed_bzip2](https://github.com/mxmlnkn/indexed_bzip2) repository. You have two options:

#### Option A: Git Submodule (Recommended)

```bash
cd rapidgzip-wrapper
git submodule add https://github.com/mxmlnkn/indexed_bzip2.git vendor/indexed_bzip2
git submodule update --init --recursive
```

#### Option B: Copy Source Files

Copy the necessary files from the indexed_bzip2 repository:

```bash
mkdir -p vendor/indexed_bzip2/src
# Copy rapidgzip source
# Download and extract from: https://github.com/mxmlnkn/indexed_bzip2/tree/master/src/rapidgzip
```

### 2. Update C++ Wrapper

Uncomment and complete the implementation in `cpp/rapidgzip_c_wrapper.cpp`:

```cpp
#include "rapidgzip.hpp"

// Replace void* reader with:
std::unique_ptr<rapidgzip::ParallelGzipReader<>> reader;
```

### 3. Update build.rs

Uncomment the include paths and dependency linking:

```rust
build.include(vendor_dir.join("indexed_bzip2/src/rapidgzip"));
build.include(vendor_dir.join("indexed_bzip2/src"));
```

### 4. Add Dependencies

The rapidgzip library requires:

- **zlib**: Standard compression library
  ```bash
  # Ubuntu/Debian
  sudo apt-get install zlib1g-dev

  # macOS
  brew install zlib
  ```

- **ISA-L** (Optional, recommended for performance): Intel Storage Acceleration Library
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libisal-dev

  # macOS
  brew install isa-l
  ```

Update `build.rs` to link these libraries:
```rust
println!("cargo:rustc-link-lib=z");
println!("cargo:rustc-link-lib=isal");  // If available
```

### 5. Handle Platform Differences

The library may require different configurations for different platforms. Consider:

- macOS uses `libc++`, Linux uses `libstdc++` (already handled in build.rs)
- Static vs dynamic linking preferences
- Feature detection for optional dependencies like ISA-L

## API Usage

Once integrated, the API will work like this:

```rust
use rapidgzip_wrapper::ParallelGzipReader;
use std::io::Read;

// Open a gzip file with automatic thread count
let mut reader = ParallelGzipReader::open("data.gz", 0)?;

// Read decompressed data
let mut buffer = vec![0u8; 4096];
let bytes_read = reader.read(&mut buffer)?;

// Seek is also supported
use std::io::Seek;
reader.seek(std::io::SeekFrom::Start(1000))?;
```

## Configuration Options

The `ParallelGzipReader` supports:

- **Thread count**: Specify number of threads (0 = auto-detect)
- **CRC32 verification**: Enable/disable checksum validation
- **Size queries**: Get decompressed size (when available)

## Performance Notes

Rapidgzip can provide significant speedups:

- Up to **20-30x faster** than single-threaded gzip on multi-core systems
- Speedup depends on available CPU cores and data characteristics
- Best for large files where parallelization overhead is amortized

## Testing

Once integrated, add tests:

```bash
cargo test
```

Consider adding:

- Unit tests for basic read/seek operations
- Integration tests with real gzip files
- Benchmark comparisons with standard gzip
- Edge case tests (empty files, corrupted data, etc.)

## License

This wrapper is licensed under MIT (same as mbf-fastq-processor).

The rapidgzip library itself is licensed under Apache-2.0 or MIT (dual-licensed).
Check the [rapidgzip repository](https://github.com/mxmlnkn/rapidgzip) for details.

## References

- [rapidgzip repository](https://github.com/mxmlnkn/rapidgzip)
- [indexed_bzip2 repository](https://github.com/mxmlnkn/indexed_bzip2)
- [rapidgzip Python package](https://pypi.org/project/rapidgzip/)
