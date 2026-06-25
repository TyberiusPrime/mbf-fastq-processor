use anyhow::{Context, Result, bail};
use ex::Wrapper;
use schemars::JsonSchema;
use std::num::NonZero;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Read, path::Path};
use toml_pretty_deser::prelude::*;

use crate::io::parsers::{self, ThreadCount};
use crate::io::reads::SegmentsCombined;
use crate::{CompressionFormat, STDIN_MAGIC_PATH};
use fastqrab_config::{default_comment_insert_char, tpd_adapt_u8_from_byte_or_char};

pub enum InputFile {
    Fastq(ex::fs::File, Option<PathBuf>),
    Fasta(ex::fs::File, Option<PathBuf>),
    Bam(ex::fs::File, Option<PathBuf>),
}

#[derive(Copy, Clone)]
pub enum DecompressionOptions {
    Default,
    /// Decode out-of-process via the sibling `fastqrab-decompressor` binary
    /// (gzip *or* zstd). `thread_count` sizes the gzip decode pool; it is ignored
    /// for zstd (serial libzstd decode).
    Subprocess {
        thread_count: ThreadCount,
    },
}

/// Which decoder the out-of-process decompressor should run, passed to the child
/// as `--format`. Only the formats the child can decode are representable.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DecompressorFormat {
    Gzip,
    Zstd,
}

impl DecompressorFormat {
    fn as_arg(self) -> &'static str {
        match self {
            DecompressorFormat::Gzip => "gzip",
            DecompressorFormat::Zstd => "zstd",
        }
    }

    /// Map niffler's detected format to a child format, when the child supports
    /// it (gzip / zstd). Other formats (bzip2, xz, …) return `None` and stay on
    /// the in-process niffler decoder.
    #[must_use]
    pub fn from_niffler(format: niffler::send::compression::Format) -> Option<Self> {
        match format {
            niffler::send::compression::Format::Gzip => Some(DecompressorFormat::Gzip),
            niffler::send::compression::Format::Zstd => Some(DecompressorFormat::Zstd),
            _ => None,
        }
    }

    /// The niffler format this child format corresponds to, for the downstream
    /// read-count `bytes_per_base` heuristics (which key off compression).
    #[must_use]
    pub fn to_niffler(self) -> niffler::send::compression::Format {
        match self {
            DecompressorFormat::Gzip => niffler::send::compression::Format::Gzip,
            DecompressorFormat::Zstd => niffler::send::compression::Format::Zstd,
        }
    }
}

#[derive(serde::Serialize, Clone, PartialEq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct InputOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub fasta_fake_quality: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bam_include_mapped: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bam_include_unmapped: Option<bool>,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char", alias = "read_comment_char")]
    pub read_comment_character: u8,

    #[serde(skip_serializing)]
    pub use_rapidgzip: bool,

    pub threads_per_segment: Option<usize>,
}

impl PartialInputOptions {
    /// Enable/disable rapidgzip. defaults to enabled if we can find our sibling
    /// decompressor binary.
    fn configure_rapid_gzip(&mut self) {
        use crate::io::input::find_decompressor;
        match &self.use_rapidgzip.state {
            TomlValueState::Missing { .. } => {
                // auto detect
                self.use_rapidgzip.state = TomlValueState::Ok;
                self.use_rapidgzip.value = Some(find_decompressor().is_some());
            }
            TomlValueState::Ok => {
                if *self.use_rapidgzip.value.as_ref().expect("State was ok")
                    && find_decompressor().is_none()
                {
                    self.use_rapidgzip.state = TomlValueState::ValidationFailed {
                        message: "rapidgzip requested but the decompressor binary was not found"
                            .to_string(),
                    };
                    self.use_rapidgzip.help = Some(
                                "The 'fastqrab-decompressor' binary must sit next to the fastqrab binary. Set use_rapidgzip to false (or leave off for auto-detect) if it is unavailable.".to_string(),
                            );
                }
            }
            _ => {} // cov:excl-line
        }
    }
}

impl<R> VerifyIn<R> for PartialInputOptions {
    fn verify(
        &mut self,
        _parent: &R,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.fasta_fake_quality.verify(|opt_v| {
            if let Some(v) = opt_v {
                if *v >= 33 && *v <= 126 {
                    Ok(())
                } else {
                    Err(ValidationFailure::new(
                        "Out of PHRED range (33..126)",
                        Some("'B' might be a good value"),
                    ))
                }
            } else {
                Ok(())
            }
        });
        self.read_comment_character
            .or_with(default_comment_insert_char);

        self.configure_rapid_gzip();

        Ok(())
    }
}

impl Default for InputOptions {
    fn default() -> Self {
        InputOptions {
            fasta_fake_quality: None,
            bam_include_mapped: None,
            bam_include_unmapped: None,
            read_comment_character: default_comment_insert_char(),
            use_rapidgzip: false,
            threads_per_segment: None,
        }
    }
}
impl InputFile {
    /// Build an [`InputFile`] from an already-open, seekable handle that has no
    /// associated path — used for a step's declared auxiliary input files, which
    /// the runtime opens up front (see `StepInputFiles`). The read format is
    /// sniffed from the handle's (possibly compressed) magic bytes. rapidgzip
    /// needs a path, so these inputs always use the default decompression.
    ///
    /// # Errors
    /// On io errors or an undecidable file format.
    pub fn from_handle(mut file: ex::fs::File) -> Result<InputFile> {
        let format = detect_input_format_from_handle(&mut file)?;
        Ok(match format {
            DetectedInputFormat::Fastq => InputFile::Fastq(file, None),
            DetectedInputFormat::Fasta => InputFile::Fasta(file, None),
            DetectedInputFormat::Bam => InputFile::Bam(file, None),
        })
    }

    #[mutants::skip] // will just fall back to default decompression options, which obvs. works
    #[must_use]
    pub fn get_filename(&self) -> Option<&PathBuf> {
        match self {
            InputFile::Fastq(_, filename)
            | InputFile::Fasta(_, filename)
            | InputFile::Bam(_, filename) => filename.as_ref(),
        }
    }

    /// # Errors
    /// On invalid options for this file type.
    pub fn get_parser(
        self,
        target_reads_per_block: NonZero<usize>,
        buffer_size: usize,
        thread_counts: parsers::ParserThreadCounts,
        options: &crate::io::input::InputOptions,
    ) -> Result<Box<dyn parsers::Parser>> {
        // Decompression (rapidgzip / bgzf) and the pod-parser demux pool are
        // sized separately upstream; route each to its own consumer.
        let thread_count = thread_counts.decompression;
        let decompression_options = if options.use_rapidgzip && self.get_filename().is_some() {
            DecompressionOptions::Subprocess { thread_count }
        } else {
            DecompressionOptions::Default
        };
        match self {
            InputFile::Fastq(file, filename) => Ok(Box::new(parsers::PodFastqParser::new(
                file.into_inner(),
                filename.as_ref(),
                target_reads_per_block,
                buffer_size,
                thread_counts.pod_demux.0.get(),
                decompression_options,
            )?)), // cov:excl-line
            InputFile::Fasta(file, filename) => {
                let fake_quality = options
                    .fasta_fake_quality
                    .context("input.options.fasta_fake_quality must be set for FASTA inputs")?;
                let parser = parsers::FastaParser::new(
                    file,
                    filename.as_ref(),
                    target_reads_per_block,
                    fake_quality,
                    decompression_options,
                )?; // cov:excl-line
                Ok(Box::new(parser))
            }
            InputFile::Bam(file, path) => {
                let include_mapped = options
                    .bam_include_mapped
                    .context("input.options.bam_include_mapped must be set for BAM inputs")?;
                let include_unmapped = options
                    .bam_include_unmapped
                    .context("input.options.bam_include_unmapped must be set for BAM inputs")?;
                let parser = parsers::BamParser::new(
                    file,
                    path,
                    target_reads_per_block,
                    include_mapped,
                    include_unmapped,
                    thread_count.0,
                )?; // cov:excl-line
                Ok(Box::new(parser))
            }
        }
    }
}

pub struct InputFiles {
    pub segment_files: SegmentsCombined<Vec<InputFile>>,
    pub total_size_of_largest_segment: Option<u64>,
    pub largest_segment_idx: usize,
}

#[must_use]
pub fn total_file_size(readers: &Vec<InputFile>) -> Option<u64> {
    let mut total = 0;
    for reader in readers {
        let file = match &reader {
            InputFile::Fastq(f, __opt_filename) => f,
            InputFile::Fasta(f, _opt_filename) => f,
            InputFile::Bam(f, _) => f,
        };
        match file.metadata() {
            Ok(metadata) => {
                total += metadata.len();
            }
            Err(_) => {
                return None; // cov:excl-line
            }
        }
    }
    Some(total)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedInputFormat {
    Fastq,
    Fasta,
    Bam,
}

/// # Errors
/// On io errors, on invalid / undecidable file types
pub fn detect_input_format(path: &Path) -> Result<(DetectedInputFormat, CompressionFormat)> {
    if path == Path::new(STDIN_MAGIC_PATH) {
        return Ok((DetectedInputFormat::Fastq, CompressionFormat::Uncompressed));
    }
    #[cfg(unix)]
    if let Ok(metadata) = std::fs::metadata(path) {
        //this is a band aid.
        {
            use std::os::unix::fs::FileTypeExt;
            if metadata.file_type().is_fifo() {
                return Ok((DetectedInputFormat::Fastq, CompressionFormat::Uncompressed));
            }
        }
    }

    let file = open_file(path)?;
    let (mut reader, format) =
        niffler::send::get_reader(Box::new(file)).context("Problem detecting file format")?;
    let mut buf = [0u8; 4];
    let bytes_read = reader.read(&mut buf)?;
    if bytes_read >= 4 && &buf[..4] == b"BAM\x01" {
        return Ok((DetectedInputFormat::Bam, CompressionFormat::Uncompressed));
    }
    let compression_format = match format {
        niffler::send::compression::Format::Gzip => CompressionFormat::Gzip,
        niffler::send::compression::Format::Zstd => CompressionFormat::Zstd,
        niffler::send::compression::Format::No => CompressionFormat::Uncompressed,
        _ => bail!("Unsupported compression format for input detection"),
    };
    if bytes_read >= 1 {
        match buf[0] {
            b'>' => Ok((DetectedInputFormat::Fasta, compression_format)),
            b'@' => Ok((DetectedInputFormat::Fastq, compression_format)),
            _ => {
                bail!(
                    "Could not detect input format for {path}. Expected FASTA, FASTQ, or BAM.",
                    path = path.display()
                );
            }
        }
    } else {
        // an empty file. We just treat it as no reads fastq and let the downstream handle
        // 0 reads gracefully
        Ok((DetectedInputFormat::Fastq, compression_format))
    }
}

/// Detect the read format of a seekable input handle by sniffing its (possibly
/// compressed) magic bytes, then rewind it so the caller reads from the start.
/// The handle-based twin of [`detect_input_format`], for declared auxiliary
/// inputs that carry no path. Compression is auto-detected by the parser later,
/// so only the read format is returned.
///
/// # Errors
/// On io errors or an undecidable file format.
fn detect_input_format_from_handle(file: &mut ex::fs::File) -> Result<DetectedInputFormat> {
    use std::io::{Seek, SeekFrom};
    // Sniff through a clone so niffler can consume bytes; `try_clone` shares the
    // OS file offset on unix, so we rewind the original afterwards regardless.
    let clone = file.try_clone()?;
    let (mut reader, _format) =
        niffler::send::get_reader(Box::new(clone)).context("Problem detecting file format")?;
    let mut buf = [0u8; 4];
    let bytes_read = reader.read(&mut buf)?;
    drop(reader);
    file.seek(SeekFrom::Start(0))?;
    if bytes_read >= 4 && &buf[..4] == b"BAM\x01" {
        return Ok(DetectedInputFormat::Bam);
    }
    if bytes_read >= 1 {
        match buf[0] {
            b'>' => Ok(DetectedInputFormat::Fasta),
            b'@' => Ok(DetectedInputFormat::Fastq),
            _ => bail!("Could not detect input format from handle. Expected FASTA, FASTQ, or BAM."),
        }
    } else {
        // an empty file: treat as a no-read fastq, matching `detect_input_format`.
        Ok(DetectedInputFormat::Fastq)
    }
}

pub fn open_text_file(maybe_compressed_filename: impl AsRef<Path>) -> Result<Box<dyn Read + Send>> {
    let in_stream = open_file(maybe_compressed_filename)?;
    let (reader, _format) =
        niffler::send::get_reader(Box::new(in_stream)).context("Problem detecting file format")?;
    Ok(reader)
}

/// # Errors
///
/// When the file can't be opened
pub fn open_file(filename: impl AsRef<Path>) -> Result<ex::fs::File> {
    let fh = ex::fs::File::open(filename.as_ref()).context(format!(
        "Could not open file \"{}\"",
        filename.as_ref().display()
    ))?;
    Ok(fh)
}

pub fn open_input_file(filename: impl AsRef<Path>) -> Result<InputFile> {
    let filename = filename.as_ref();
    if filename.to_string_lossy() == STDIN_MAGIC_PATH {
        let file = open_stdin()?;
        return Ok(InputFile::Fastq(file, None));
    }
    let path = Path::new(filename);
    let format = detect_input_format(path)?.0;

    let file = open_file(path)?;
    let input_file = match format {
        DetectedInputFormat::Fastq => InputFile::Fastq(file, Some(path.to_owned())),
        DetectedInputFormat::Fasta => InputFile::Fasta(file, Some(path.to_owned())),
        DetectedInputFormat::Bam => InputFile::Bam(file, Some(path.to_owned())),
    };
    Ok(input_file)
}

fn open_stdin() -> Result<ex::fs::File> {
    #[cfg(unix)]
    {
        use anyhow::Context as _;
        ex::fs::File::open("/dev/stdin").context("Failed to access stdin via /dev/stdin")
    }
    #[cfg(windows)]
    {
        bail!("Stdin input is not supported on windows. PRs welcome");
    }
    #[cfg(not(any(unix, windows)))]
    {
        bail!(
            "(input): '{STDIN_MAGIC_PATH}' is not supported on this platform (unknown stdio semantics)."
        );
    }
}

/// Locate our out-of-process decompressor: the `fastqrab-decompressor` crate's
/// binary, which must sit right next to the running fastqrab binary. The name is
/// derived by suffixing the running executable's file name with `-decompressor`,
/// so a renamed `fastqrab_0.9.1` looks for a sibling `fastqrab_0.9.1-decompressor`.
///
/// The `FASTQRAB_DECOMPRESSOR` environment variable overrides this lookup with an
/// explicit path. This keeps integration tests hermetic (they can point at a
/// freshly built binary regardless of target-dir layout) and lets packagers
/// relocate the decompressor away from the main binary.
#[must_use]
pub fn find_decompressor() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FASTQRAB_DECOMPRESSOR") {
        let path = PathBuf::from(path);
        return path.exists().then_some(path);
    }
    let current = std::env::current_exe().ok()?;
    let dir = current.parent()?;
    let name = current.file_name()?.to_str()?;
    let candidate = dir.join(format!("{name}-decompressor"));
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

/// Open a (possibly compressed) byte stream for a FASTQ/FASTA input, applying
/// rapidgzip when requested (and the detected format is gzip), otherwise letting
/// niffler pick the decompressor. Returns the decompressed reader together with
/// the detected compression format (used downstream for the read-count
/// estimation heuristics).
///
/// This is the shared core behind both the legacy [`FastqParser`] and the
/// columnar pod parser.
///
/// [`FastqParser`]: crate::io::parsers::FastqParser
///
/// # Errors
/// On io errors or when spawning rapidgzip fails.
///
/// # Panics
/// If rapidgzip is requested for a path-less (stdin) input — validation prevents
/// this combination upstream.
pub fn open_decompressed_reader(
    file: std::fs::File,
    filename: Option<&PathBuf>,
    decompression_options: DecompressionOptions,
) -> Result<(Box<dyn Read + Send>, niffler::send::compression::Format)> {
    let (mut reader, format) = niffler::send::get_reader(Box::new(file))?;
    if let DecompressionOptions::Subprocess { thread_count } = decompression_options
        && let Some(child_format) = DecompressorFormat::from_niffler(format)
    {
        let file = spawn_decompressor(
            filename.expect("out-of-process decode and stdin not supported"),
            child_format,
            thread_count,
        )?; // cov:excl-line
        reader = Box::new(file);
    }
    Ok((reader, format))
}

/// Spawns our out-of-process `fastqrab-decompressor` to decode a gzip or zstd
/// file, returning its stdout as a readable file handle. Keeping the decoder in a
/// separate process isolates all of its `unsafe` from the main process.
pub fn spawn_decompressor(
    filename: &Path,
    format: DecompressorFormat,
    thread_count: ThreadCount,
) -> Result<std::fs::File> {
    #[cfg(unix)]
    {
        const PIPE_BUF_SIZE: libc::c_int = 1024 * 1024;
        use std::os::unix::io::{FromRawFd, IntoRawFd};

        let decompressor = find_decompressor()
            .context("fastqrab-decompressor binary not found next to the fastqrab binary")?;
        // Mirror the `fastqrab-decompressor` CLI: `--format`, `-P` threads, then
        // the positional input path.
        let mut cmd = Command::new(decompressor);
        cmd.arg("--format")
            .arg(format.as_arg())
            .arg("-P")
            .arg(thread_count.0.to_string())
            .arg(filename)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().context(format!(
            "Failed to spawn fastqrab-decompressor process for file: {}.",
            filename.display()
        ))?; // cov:excl-line
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture fastqrab-decompressor stdout")?;

        // Convert the stdout pipe to an ex::fs::File
        // We need to use the file descriptor directly
        let raw_fd = stdout.into_raw_fd();

        // Enlarge the pipe buffer to 1 MiB (the usual /proc/sys/fs/pipe-max-size).
        // The default 64 KiB pipe forces the 4 MiB decompressor chunks through in
        // ~64 KiB reads, so producer and consumer ping-pong on a near-empty/near-full
        // buffer instead of overlapping (millions of voluntary context switches).
        // A bigger buffer gives both sides slack and cuts read/write syscalls.
        // Best-effort: failure here is not fatal, the pipe just keeps its default size.
        // SAFETY: `raw_fd` is a live pipe fd we own; F_SETPIPE_SZ takes an int arg.
        unsafe {
            libc::fcntl(raw_fd, libc::F_SETPIPE_SZ, PIPE_BUF_SIZE);
        }

        // SAFETY: We own the file descriptor from the child process stdout
        let file = unsafe { std::fs::File::from_raw_fd(raw_fd) };
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        bail!("Out-of-process decompression is only supported on Unix systems");
    }
}

/// Magic-sniff a *regular* file for a compression format the out-of-process
/// decompressor can decode over shared memory: gzip (`1f 8b`) or zstd
/// (`28 b5 2f fd`). Peeks and rewinds, so a `None` result leaves the file
/// untouched at offset 0 for the fall-through `Read`-based path. Pipes/FIFOs (not
/// seekable) and unrecognized/short files return `None` and keep their existing
/// transport.
#[cfg(unix)]
#[must_use]
pub fn shm_eligible_format(file: &std::fs::File) -> Option<DecompressorFormat> {
    use std::io::{Read as _, Seek as _, SeekFrom};
    let meta = file.metadata().ok()?;
    if !meta.is_file() {
        return None;
    }
    let mut handle: &std::fs::File = file;
    let mut magic = [0u8; 4];
    let read = handle.read_exact(&mut magic);
    // Always rewind a regular file, even on a short read, so the fall-through
    // reader is unaffected.
    let _ = handle.seek(SeekFrom::Start(0));
    read.ok()?;
    if magic[..2] == [0x1f, 0x8b] {
        Some(DecompressorFormat::Gzip)
    } else if magic == [0x28, 0xb5, 0x2f, 0xfd] {
        Some(DecompressorFormat::Zstd)
    } else {
        None
    }
}

/// A shared-memory region mapped from the decompressor's `memfd`, unmapped on
/// drop. The pod parser keeps it alive (behind an `Arc`) for as long as any
/// borrowed-slot chunk is outstanding.
#[cfg(unix)]
pub struct ShmRegion {
    ptr: *mut u8,
    len: usize,
}

// SAFETY: the region is a plain shared byte buffer. All access is coordinated by
// the single-owner-per-slot protocol carried on the descriptor / slot-return
// pipes (not by Rust's aliasing rules), so the handle is safe to move and share
// across threads.
#[cfg(unix)]
unsafe impl Send for ShmRegion {}
// SAFETY: same invariant as `Send` above — slot ownership is coordinated by the
// control pipes, not Rust aliasing, so shared `&ShmRegion` access is sound.
#[cfg(unix)]
unsafe impl Sync for ShmRegion {}

#[cfg(unix)]
impl ShmRegion {
    /// Base pointer of the mapped region. Callers form per-slot sub-slices under
    /// the single-owner invariant (a slot is read only between receiving its
    /// descriptor and returning its id).
    #[must_use]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }
}

#[cfg(unix)]
impl Drop for ShmRegion {
    fn drop(&mut self) {
        // SAFETY: `ptr`/`len` are exactly what `mmap` returned; never remapped.
        unsafe {
            libc::munmap(self.ptr.cast(), self.len);
        }
    }
}

/// Handle to a decompressor running in shared-memory mode: the mapped region
/// plus the two control pipes and the child process.
#[cfg(unix)]
pub struct ShmDecompressor {
    /// The mapped shared region, kept alive as long as any borrowed chunk.
    pub region: std::sync::Arc<ShmRegion>,
    /// Child stdout: an ordered stream of `(slot:u32, len:u32)` descriptors,
    /// terminated by a `(u32::MAX, 0)` sentinel.
    pub descriptors: std::process::ChildStdout,
    /// Child stdin: freed slot ids (`u32` LE) returned to the decompressor.
    pub slot_return: std::process::ChildStdin,
    /// The decompressor child process (waited on at EOF to surface its exit).
    pub child: std::process::Child,
    pub slots: usize,
    pub slot_size: usize,
}

/// Spawn the out-of-process decompressor in shared-memory mode: create an
/// `memfd`-backed region of `slots × slot_size` bytes, map it `MAP_SHARED`, and
/// hand the (non-`CLOEXEC`) fd to the child by inheritance. The child memcpies
/// finished chunks into free slots and streams `(slot, len)` descriptors back on
/// its stdout; we return freed slot ids on its stdin.
///
/// The shared-memory transport is format-agnostic — `format` only selects the
/// child's decode backend (gzip/zstd); inputs that can't ride shm (FIFOs, other
/// codecs) stay on [`spawn_decompressor`]'s pipe transport.
#[cfg(unix)]
pub fn spawn_decompressor_shm(
    filename: &Path,
    format: DecompressorFormat,
    thread_count: ThreadCount,
    slots: usize,
    slot_size: usize,
) -> Result<ShmDecompressor> {
    let decompressor = find_decompressor()
        .context("fastqrab-decompressor binary not found next to the fastqrab binary")?;
    let total = slots
        .checked_mul(slot_size)
        .context("shared-memory region size overflow")?;
    let total_off = libc::off_t::try_from(total).context("shared-memory region too large")?;

    // memfd sized to the whole region. No `MFD_CLOEXEC` so the child inherits the
    // fd (with the same number) and can map the same region.
    // SAFETY: standard libc call with a valid C name and zero flags.
    let fd = unsafe { libc::memfd_create(c"fastqrab-shm".as_ptr(), 0) };
    if fd < 0 {
        bail!("memfd_create failed: {}", std::io::Error::last_os_error());
    }

    // SAFETY: `fd` is the live memfd we just created.
    if unsafe { libc::ftruncate(fd, total_off) } != 0 {
        let err = std::io::Error::last_os_error();
        // SAFETY: `fd` is ours and unused past this point.
        unsafe { libc::close(fd) };
        bail!("ftruncate of shared-memory region failed: {err}");
    }

    // SAFETY: mapping our own memfd `MAP_SHARED` for read+write.
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            total,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        let err = std::io::Error::last_os_error();
        // SAFETY: `fd` is ours and unused past this point.
        unsafe { libc::close(fd) };
        bail!("mmap of shared-memory region failed: {err}");
    }
    let region = std::sync::Arc::new(ShmRegion {
        ptr: ptr.cast::<u8>(),
        len: total,
    });

    let mut cmd = Command::new(&decompressor);
    cmd.arg("--format")
        .arg(format.as_arg())
        .arg("--shm-fd")
        .arg(fd.to_string())
        .arg("--shm-slots")
        .arg(slots.to_string())
        .arg("--shm-slot-size")
        .arg(slot_size.to_string())
        // Leave the decoder's chunk size at its default: the caller sizes slots
        // comfortably larger than a decode chunk so the common chunk fits one
        // slot, while the child still splits any oversized chunk across slots.
        .arg("-P")
        .arg(thread_count.0.to_string())
        .arg(filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn().context(format!(
        "Failed to spawn fastqrab-decompressor (shm mode) for file: {}.",
        filename.display()
    ))?;

    // The child inherited the fd at fork; our mapping holds the region alive, so
    // the parent no longer needs its own fd.
    // SAFETY: `fd` is ours and no longer referenced in the parent.
    unsafe { libc::close(fd) };

    let descriptors = child
        .stdout
        .take()
        .context("Failed to capture fastqrab-decompressor stdout (shm descriptors)")?;
    let slot_return = child
        .stdin
        .take()
        .context("Failed to capture fastqrab-decompressor stdin (shm slot returns)")?;

    Ok(ShmDecompressor {
        region,
        descriptors,
        slot_return,
        child,
        slots,
        slot_size,
    })
}
