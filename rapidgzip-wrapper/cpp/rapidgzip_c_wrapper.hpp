#ifndef RAPIDGZIP_C_WRAPPER_HPP
#define RAPIDGZIP_C_WRAPPER_HPP

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle type for the ParallelGzipReader
typedef struct RapidGzipReader RapidGzipReader;

// Error codes
typedef enum {
    RAPIDGZIP_OK = 0,
    RAPIDGZIP_ERROR_INVALID_HANDLE = -1,
    RAPIDGZIP_ERROR_OPEN_FAILED = -2,
    RAPIDGZIP_ERROR_READ_FAILED = -3,
    RAPIDGZIP_ERROR_SEEK_FAILED = -4,
    RAPIDGZIP_ERROR_EOF = -5,
    RAPIDGZIP_ERROR_UNKNOWN = -99
} RapidGzipError;

// Seek modes (similar to SEEK_SET, SEEK_CUR, SEEK_END)
typedef enum {
    RAPIDGZIP_SEEK_SET = 0,
    RAPIDGZIP_SEEK_CUR = 1,
    RAPIDGZIP_SEEK_END = 2
} RapidGzipSeekMode;

/**
 * Open a gzip file for parallel decompression
 *
 * @param filepath Path to the gzip file
 * @param num_threads Number of threads to use for parallel decompression (0 = auto)
 * @param out_reader Pointer to receive the reader handle
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_open(const char* filepath, size_t num_threads, RapidGzipReader** out_reader);

/**
 * Open a gzip file from a file descriptor
 *
 * @param fd File descriptor
 * @param num_threads Number of threads to use for parallel decompression (0 = auto)
 * @param out_reader Pointer to receive the reader handle
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_open_fd(int fd, size_t num_threads, RapidGzipReader** out_reader);

/**
 * Read decompressed data from the reader
 *
 * @param reader Reader handle
 * @param buffer Buffer to read into
 * @param size Number of bytes to read
 * @param out_bytes_read Pointer to receive the number of bytes actually read
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_read(RapidGzipReader* reader, uint8_t* buffer, size_t size, size_t* out_bytes_read);

/**
 * Seek to a position in the decompressed stream
 *
 * @param reader Reader handle
 * @param offset Offset to seek to
 * @param whence Seek mode (RAPIDGZIP_SEEK_SET, RAPIDGZIP_SEEK_CUR, or RAPIDGZIP_SEEK_END)
 * @param out_position Pointer to receive the new position (can be NULL)
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_seek(RapidGzipReader* reader, int64_t offset, RapidGzipSeekMode whence, uint64_t* out_position);

/**
 * Get the current position in the decompressed stream
 *
 * @param reader Reader handle
 * @param out_position Pointer to receive the current position
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_tell(RapidGzipReader* reader, uint64_t* out_position);

/**
 * Check if the reader has reached end-of-file
 *
 * @param reader Reader handle
 * @param out_eof Pointer to receive EOF status (1 = EOF, 0 = not EOF)
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_eof(RapidGzipReader* reader, int* out_eof);

/**
 * Enable or disable CRC32 verification
 *
 * @param reader Reader handle
 * @param enabled 1 to enable, 0 to disable
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_set_crc32_enabled(RapidGzipReader* reader, int enabled);

/**
 * Get the size of the decompressed data (if available)
 *
 * @param reader Reader handle
 * @param out_size Pointer to receive the size (may be 0 if not yet determined)
 * @return Error code (RAPIDGZIP_OK on success)
 */
int rapidgzip_size(RapidGzipReader* reader, uint64_t* out_size);

/**
 * Close the reader and free all resources
 *
 * @param reader Reader handle to close
 */
void rapidgzip_close(RapidGzipReader* reader);

#ifdef __cplusplus
}
#endif

#endif // RAPIDGZIP_C_WRAPPER_HPP
