# Integration TODO

This document tracks the remaining steps to complete the rapidgzip wrapper integration.

## Critical Next Steps

### 1. Fetch rapidgzip Source Code

**Option A: Git Submodule (Recommended)**
```bash
cd /home/user/mbf-fastq-processor/rapidgzip-wrapper
git submodule add https://github.com/mxmlnkn/indexed_bzip2.git vendor/indexed_bzip2
git submodule update --init --recursive
cd vendor/indexed_bzip2
git checkout b31e5baea41c1a7dfd130b92dec32d2dceba98fd
```

**Option B: Manual Download**
- Download the indexed_bzip2 repository at commit `b31e5baea41c1a7dfd130b92dec32d2dceba98fd`
- Extract to `vendor/indexed_bzip2/`

### 2. Update C++ Wrapper Implementation

In `cpp/rapidgzip_c_wrapper.cpp`, replace all placeholder code with actual rapidgzip API calls:

1. **Include headers** (at the top):
   ```cpp
   #include "../vendor/indexed_bzip2/src/rapidgzip/rapidgzip.hpp"
   ```

2. **Update RapidGzipReader struct**:
   ```cpp
   struct RapidGzipReader {
       std::unique_ptr<rapidgzip::ParallelGzipReader<>> reader;
       bool eof_reached;

       RapidGzipReader() : reader(nullptr), eof_reached(false) {}
       ~RapidGzipReader() = default;
   };
   ```

3. **Implement rapidgzip_open**:
   ```cpp
   reader->reader = std::make_unique<rapidgzip::ParallelGzipReader<>>(
       filepath,
       num_threads == 0 ? 0 : static_cast<size_t>(num_threads)
   );
   ```

4. **Implement rapidgzip_open_fd**:
   ```cpp
   reader->reader = std::make_unique<rapidgzip::ParallelGzipReader<>>(
       fd,
       num_threads == 0 ? 0 : static_cast<size_t>(num_threads)
   );
   ```

5. **Implement rapidgzip_read**:
   ```cpp
   size_t bytes_read = reader->reader->read(
       reinterpret_cast<char*>(buffer),
       size
   );
   ```

6. **Implement rapidgzip_seek**:
   ```cpp
   int seek_mode = (whence == RAPIDGZIP_SEEK_SET) ? SEEK_SET :
                   (whence == RAPIDGZIP_SEEK_CUR) ? SEEK_CUR : SEEK_END;
   size_t new_pos = reader->reader->seek(offset, seek_mode);
   ```

7. **Implement rapidgzip_tell**:
   ```cpp
   *out_position = reader->reader->tell();
   ```

8. **Implement rapidgzip_eof**:
   ```cpp
   *out_eof = reader->reader->eof() ? 1 : 0;
   ```

9. **Implement rapidgzip_set_crc32_enabled**:
   ```cpp
   reader->reader->setCRC32Enabled(enabled != 0);
   ```

10. **Implement rapidgzip_size**:
    ```cpp
    auto size_opt = reader->reader->size();
    *out_size = size_opt.value_or(0);
    ```

### 3. Update build.rs

Uncomment and verify the include paths:

```rust
build.include(vendor_dir.join("indexed_bzip2/src/rapidgzip"));
build.include(vendor_dir.join("indexed_bzip2/src"));
```

Check if additional dependencies need to be linked:
```rust
// Check if these are needed based on rapidgzip's requirements:
println!("cargo:rustc-link-lib=z");         // zlib
// println!("cargo:rustc-link-lib=isal");   // ISA-L (optional, for performance)
```

### 4. Install System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install -y zlib1g-dev
# Optional but recommended for performance:
sudo apt-get install -y nasm libisal-dev
```

**macOS:**
```bash
brew install zlib
# Optional:
brew install nasm isa-l
```

### 5. Test the Build

```bash
cd /home/user/mbf-fastq-processor
cargo build -p rapidgzip-wrapper
```

If successful, you should see the library compile without errors.

### 6. Create Test Files

Create a simple test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::fs::File;

    #[test]
    fn test_decompress_simple() {
        // Create a test gzip file
        let test_data = b"Hello, World! This is a test.";
        let mut gz_file = tempfile::NamedTempFile::new().unwrap();

        // Write gzipped data
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(test_data).unwrap();
        let compressed = encoder.finish().unwrap();
        gz_file.write_all(&compressed).unwrap();

        // Read it back with rapidgzip
        let mut reader = ParallelGzipReader::open(gz_file.path(), 0).unwrap();
        let mut decompressed = Vec::new();
        reader.read_to_end(&mut decompressed).unwrap();

        assert_eq!(&decompressed, test_data);
    }
}
```

### 7. Performance Benchmarks (Optional)

Create benchmarks to validate the parallel speedup:

```bash
# Add to Cargo.toml:
# [[bench]]
# name = "decompress_bench"
# harness = false
#
# [dev-dependencies]
# criterion = "0.5"
```

## Troubleshooting

### Build Errors

1. **Missing rapidgzip headers**: Verify vendor/indexed_bzip2 exists and contains src/rapidgzip/
2. **C++17 not supported**: Update your compiler (GCC 7+, Clang 5+)
3. **Linking errors**: Check that zlib-dev is installed

### Runtime Errors

1. **Segmentation fault**: Check that file paths are valid and files exist
2. **Null pointer errors**: Verify rapidgzip reader was created successfully
3. **CRC errors**: May indicate corrupted gzip file or library issue

## Integration Checklist

- [ ] Fetch rapidgzip source code to vendor/
- [ ] Update C++ wrapper implementation (10 functions)
- [ ] Update build.rs include paths
- [ ] Install system dependencies (zlib, optionally ISA-L)
- [ ] Verify build succeeds: `cargo build -p rapidgzip-wrapper`
- [ ] Write basic tests
- [ ] Test with real gzip files
- [ ] Add to mbf-fastq-processor integration (if needed)
- [ ] Document API in rustdoc
- [ ] Create examples/
- [ ] Performance benchmarks

## Expected Timeline

- **Initial setup** (steps 1-5): 1-2 hours
- **Testing and validation** (step 6): 1-2 hours
- **Documentation and polish**: 1 hour

**Total**: 3-5 hours for a complete integration
