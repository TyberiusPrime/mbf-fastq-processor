#include "rapidgzip_c_wrapper.hpp"
#include <memory>
#include <exception>
#include <cstring>

// Include rapidgzip headers
#include "../vendor/indexed_bzip2/src/rapidgzip/rapidgzip.hpp"
#include "../vendor/indexed_bzip2/src/filereader/Standard.hpp"

// Opaque struct that holds the C++ reader
struct RapidGzipReader {
    std::unique_ptr<rapidgzip::ParallelGzipReader<>> reader;

    RapidGzipReader() : reader(nullptr) {}
    ~RapidGzipReader() = default;
};

int rapidgzip_open(const char* filepath, size_t num_threads, RapidGzipReader** out_reader) {
    if (!filepath || !out_reader) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        auto wrapper = new RapidGzipReader();

        // Create a StandardFileReader and pass it to ParallelGzipReader
        auto fileReader = std::make_unique<rapidgzip::StandardFileReader>(filepath);
        wrapper->reader = std::make_unique<rapidgzip::ParallelGzipReader<>>(
            std::move(fileReader),
            num_threads  // 0 means auto-detect
        );

        *out_reader = wrapper;
        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_OPEN_FAILED;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_open_fd(int fd, size_t num_threads, RapidGzipReader** out_reader) {
    if (fd < 0 || !out_reader) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        auto wrapper = new RapidGzipReader();

        // Create a StandardFileReader from file descriptor
        auto fileReader = std::make_unique<rapidgzip::StandardFileReader>(fd);
        wrapper->reader = std::make_unique<rapidgzip::ParallelGzipReader<>>(
            std::move(fileReader),
            num_threads  // 0 means auto-detect
        );

        *out_reader = wrapper;
        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_OPEN_FAILED;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_read(RapidGzipReader* reader, uint8_t* buffer, size_t size, size_t* out_bytes_read) {
    if (!reader || !reader->reader || !buffer || !out_bytes_read) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        size_t bytes_read = reader->reader->read(reinterpret_cast<char*>(buffer), size);
        *out_bytes_read = bytes_read;

        // EOF is indicated by read returning 0
        if (bytes_read == 0) {
            return RAPIDGZIP_ERROR_EOF;
        }

        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_READ_FAILED;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_seek(RapidGzipReader* reader, int64_t offset, RapidGzipSeekMode whence, uint64_t* out_position) {
    if (!reader || !reader->reader) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        int seek_mode = (whence == RAPIDGZIP_SEEK_SET) ? SEEK_SET :
                        (whence == RAPIDGZIP_SEEK_CUR) ? SEEK_CUR : SEEK_END;

        size_t new_pos = reader->reader->seek(offset, seek_mode);

        if (out_position) {
            *out_position = new_pos;
        }

        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_SEEK_FAILED;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_tell(RapidGzipReader* reader, uint64_t* out_position) {
    if (!reader || !reader->reader || !out_position) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        *out_position = reader->reader->tell();
        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_eof(RapidGzipReader* reader, int* out_eof) {
    if (!reader || !reader->reader || !out_eof) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        *out_eof = reader->reader->eof() ? 1 : 0;
        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_set_crc32_enabled(RapidGzipReader* reader, int enabled) {
    if (!reader || !reader->reader) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        reader->reader->setCRC32Enabled(enabled != 0);
        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

int rapidgzip_size(RapidGzipReader* reader, uint64_t* out_size) {
    if (!reader || !reader->reader || !out_size) {
        return RAPIDGZIP_ERROR_INVALID_HANDLE;
    }

    try {
        auto size_opt = reader->reader->size();
        *out_size = size_opt.value_or(0);
        return RAPIDGZIP_OK;
    } catch (const std::exception& e) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    } catch (...) {
        return RAPIDGZIP_ERROR_UNKNOWN;
    }
}

void rapidgzip_close(RapidGzipReader* reader) {
    if (reader) {
        try {
            delete reader;
        } catch (...) {
            // Suppress all exceptions in cleanup
        }
    }
}
