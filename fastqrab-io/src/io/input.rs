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

    /// Map a sniffed compression format to a child decoder format. Only the
    /// formats the child can decode are representable; [`CompressionFormat::Uncompressed`]
    /// returns `None` (no subprocess — read the bytes directly).
    #[must_use]
    pub fn from_compression(format: CompressionFormat) -> Option<Self> {
        match format {
            CompressionFormat::Gzip => Some(DecompressorFormat::Gzip),
            CompressionFormat::Zstd => Some(DecompressorFormat::Zstd),
            CompressionFormat::Uncompressed => None,
        }
    }

    /// The compression format this child format corresponds to, for the
    /// downstream read-count `bytes_per_base` heuristics (which key off
    /// compression).
    #[must_use]
    pub fn to_compression(self) -> CompressionFormat {
        match self {
            DecompressorFormat::Gzip => CompressionFormat::Gzip,
            DecompressorFormat::Zstd => CompressionFormat::Zstd,
        }
    }
}

/// Sniff a compression format from a file's leading magic bytes: gzip (`1f 8b`)
/// or zstd (`28 b5 2f fd`); anything else (including a short read) is treated as
/// uncompressed. The single source of truth for "what codec is this", replacing
/// niffler's detection so all real decoding stays in the out-of-process
/// containment zone.
#[must_use]
pub fn sniff_compression(magic: &[u8]) -> CompressionFormat {
    if magic.len() >= 2 && magic[..2] == [0x1f, 0x8b] {
        CompressionFormat::Gzip
    } else if magic.len() >= 4 && magic[..4] == [0x28, 0xb5, 0x2f, 0xfd] {
        CompressionFormat::Zstd
    } else {
        CompressionFormat::Uncompressed
    }
}

/// Read up to four leading bytes from `reader` for magic sniffing, returning them
/// alongside a reader that still yields the full original stream (the peeked bytes
/// chained back in front). Works for non-seekable inputs (pipes, stdin), unlike a
/// read-then-rewind.
fn peek_magic(mut reader: Box<dyn Read + Send>) -> Result<([u8; 4], usize, Box<dyn Read + Send>)> {
    let mut magic = [0u8; 4];
    let mut filled = 0;
    while filled < magic.len() {
        let n = reader.read(&mut magic[filled..])?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    let prefix = std::io::Cursor::new(magic[..filled].to_vec());
    Ok((magic, filled, Box::new(prefix.chain(reader))))
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

/// Bounded compressed prefix pulled into memory to peek a path-less handle's first
/// decoded bytes (see [`detect_input_format_from_handle`]). Comfortably larger
/// than a gzip/zstd block, so it always decodes well past the first record marker.
const DETECT_COMPRESSED_PREFIX: usize = 128 * 1024;

/// Read as many bytes as possible into `buf` (handling short reads), returning the
/// number filled. `Ok(0)` short of `buf.len()` means end of stream.
fn read_full(reader: &mut impl Read, buf: &mut [u8]) -> Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        let n = reader.read(&mut buf[filled..])?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    Ok(filled)
}

/// Read up to four leading bytes (the format-discriminating head) from `reader`.
fn read_head(reader: &mut impl Read) -> Result<([u8; 4], usize)> {
    let mut head = [0u8; 4];
    let n = read_full(reader, &mut head)?;
    Ok((head, n))
}

/// Recognize compression formats fastqrab deliberately does not support (bzip2,
/// xz) from their raw magic, so detection can give a precise "unsupported
/// compression" error instead of a confusing "not FASTA/FASTQ/BAM".
fn is_unsupported_compression(magic: &[u8]) -> bool {
    magic.starts_with(&[0x42, 0x5a, 0x68])                  // bzip2 "BZh"
        || magic.starts_with(&[0xfd, 0x37, 0x7a, 0x58]) // xz
}

/// Classify the leading *decoded* bytes of an input as BAM / FASTA / FASTQ.
/// `None` means they match no supported format. An empty head is treated as an
/// empty FASTQ (downstream handles zero reads gracefully).
fn classify_head(head: &[u8]) -> Option<DetectedInputFormat> {
    if head.len() >= 4 && head[..4] == *b"BAM\x01" {
        return Some(DetectedInputFormat::Bam);
    }
    match head.first() {
        Some(b'>') => Some(DetectedInputFormat::Fasta),
        // `@` is FASTQ; an empty head is treated as an empty FASTQ.
        Some(b'@') | None => Some(DetectedInputFormat::Fastq),
        Some(_) => None,
    }
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

    let file = open_file(path)?.into_inner();
    // Sniff the codec from raw magic (no decode), then read the first *decoded*
    // bytes — through the out-of-process decompressor for gzip/zstd — to tell
    // BAM / FASTA / FASTQ apart.
    let (magic, magic_n, reader) = peek_magic(Box::new(file))?;
    if is_unsupported_compression(&magic[..magic_n]) {
        bail!("Unsupported compression format for input detection");
    }
    let compression = sniff_compression(&magic[..magic_n]);
    let (head, head_n) = match DecompressorFormat::from_compression(compression) {
        Some(child_format) => {
            let mut dec =
                spawn_decompressor(path, child_format, ThreadCount(NonZero::<usize>::MIN))?;
            read_head(&mut dec)?
            // `dec` drops here, reaping the decompressor child.
        }
        None => {
            let mut reader = reader;
            read_head(&mut reader)?
        }
    };
    let format = classify_head(&head[..head_n]).with_context(|| {
        format!(
            "Could not detect input format for {}. Expected FASTA, FASTQ, or BAM.",
            path.display()
        )
    })?;
    // BAM payloads are BGZF, but their bgzf is decoded in-process by noodles, so
    // report BAM as uncompressed (matching the historical detection contract).
    let reported = if format == DetectedInputFormat::Bam {
        CompressionFormat::Uncompressed
    } else {
        compression
    };
    Ok((format, reported))
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
    // `try_clone` shares the OS file offset on unix, so read a bounded prefix from
    // the clone sequentially and rewind the original afterwards. For compressed
    // handles we decode that in-memory prefix (no path to hand the child, and no
    // concurrent fd access), which is enough to reach the first record marker.
    let mut clone = file.try_clone()?;
    let (magic, magic_n) = read_head(&mut clone)?;
    if is_unsupported_compression(&magic[..magic_n]) {
        file.seek(SeekFrom::Start(0))?;
        bail!("Unsupported compression format for input detection");
    }
    let compression = sniff_compression(&magic[..magic_n]);
    let (head, head_n) = match DecompressorFormat::from_compression(compression) {
        None => (magic, magic_n),
        Some(child_format) => {
            let mut rest = vec![0u8; DETECT_COMPRESSED_PREFIX];
            let got = read_full(&mut clone, &mut rest)?;
            let mut prefix = Vec::with_capacity(magic_n + got);
            prefix.extend_from_slice(&magic[..magic_n]);
            prefix.extend_from_slice(&rest[..got]);
            let mut dec = spawn_decompressor_from_reader(
                Box::new(std::io::Cursor::new(prefix)),
                child_format,
                ThreadCount(NonZero::<usize>::MIN),
            )?;
            read_head(&mut dec)?
            // `dec` drops here, reaping the child and ending the feeder thread.
        }
    };
    drop(clone);
    file.seek(SeekFrom::Start(0))?;
    classify_head(&head[..head_n])
        .context("Could not detect input format from handle. Expected FASTA, FASTQ, or BAM.")
}

/// Open a (possibly gzip/zstd-compressed) text file by path, returning a reader
/// over its decoded bytes. Compressed inputs decode out-of-process (all FFI in the
/// containment zone); uncompressed inputs are read directly.
pub fn open_text_file(maybe_compressed_filename: impl AsRef<Path>) -> Result<Box<dyn Read + Send>> {
    let path = maybe_compressed_filename.as_ref();
    let file = open_file(path)?.into_inner();
    let (magic, magic_n, reader) = peek_magic(Box::new(file))?;
    match DecompressorFormat::from_compression(sniff_compression(&magic[..magic_n])) {
        Some(child_format) => Ok(Box::new(spawn_decompressor(
            path,
            child_format,
            ThreadCount(NonZero::<usize>::MIN),
        )?)),
        None => Ok(reader),
    }
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

/// Open a (possibly compressed) byte stream for a FASTQ/FASTA input. gzip/zstd is
/// sniffed by magic and decoded out-of-process (all FFI in the containment zone);
/// uncompressed bytes pass straight through. Returns the decompressed reader
/// together with the detected compression format (used downstream for the
/// read-count estimation heuristics).
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
) -> Result<(Box<dyn Read + Send>, CompressionFormat)> {
    let (magic, _filled, reader) = peek_magic(Box::new(file))?;
    let compression = sniff_compression(&magic);
    let Some(child_format) = DecompressorFormat::from_compression(compression) else {
        // Uncompressed: hand the bytes straight through, no subprocess needed.
        return Ok((reader, CompressionFormat::Uncompressed));
    };
    // Compressed: decode out-of-process — all gzip/zstd FFI stays in the child.
    // `thread_count` only sizes the gzip decode pool; the Default path (stdin /
    // no explicit subprocess request) decodes serially.
    let thread_count = match decompression_options {
        DecompressionOptions::Subprocess { thread_count } => thread_count,
        DecompressionOptions::Default => ThreadCount(NonZero::<usize>::MIN),
    };
    let decoded: Box<dyn Read + Send> = match filename {
        Some(path) => Box::new(spawn_decompressor(path, child_format, thread_count)?),
        None => Box::new(spawn_decompressor_from_reader(
            reader,
            child_format,
            thread_count,
        )?),
    };
    Ok((decoded, compression))
}

/// A reader over a spawned `fastqrab-decompressor` child in pipe mode: yields the
/// child's decoded stdout and reaps the child on drop. Reaping matters because
/// callers may stop early (format detection peeks only the first bytes) — drop
/// then kills the child instead of leaving it blocked writing into a closed pipe,
/// and waits it so we don't leak a zombie.
pub struct DecompressorReader {
    stdout: std::process::ChildStdout,
    child: std::process::Child,
    /// For path-less inputs: the thread copying our input into the child's stdin.
    /// Killing the child on drop closes its stdin, so this thread finishes on its
    /// own; we don't join it.
    _feeder: Option<std::thread::JoinHandle<()>>,
    /// Set once we've reached EOF and verified the child's exit, so a reader that
    /// keeps calling `read` past EOF doesn't re-`wait` an already-reaped child.
    exit_verified: bool,
}

impl Read for DecompressorReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.stdout.read(buf)?;
        if n == 0 && !self.exit_verified {
            // EOF: the child closed stdout. Verify it succeeded — a non-zero exit
            // (e.g. a truncated/corrupt input) must surface as a read error, not a
            // silently short stream. (Detection peeks only the first bytes and
            // drops us before EOF, so this never fires for those.)
            self.exit_verified = true;
            let status = self.child.wait()?;
            if !status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("fastqrab-decompressor exited unsuccessfully: {status}"),
                ));
            }
        }
        Ok(n)
    }
}

impl Drop for DecompressorReader {
    fn drop(&mut self) {
        // Best-effort: if the consumer drained to EOF the child has already
        // exited and `kill` is a no-op; if it stopped early, this unblocks it.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Build the `fastqrab-decompressor` command shared by the path and stdin
/// spawners: `--format`, `-P` threads, positional input (`-` ⇒ stdin), stdout
/// piped, stderr inherited.
fn decompressor_command(
    input: &std::ffi::OsStr,
    format: DecompressorFormat,
    thread_count: ThreadCount,
) -> Result<Command> {
    let decompressor = find_decompressor()
        .context("fastqrab-decompressor binary not found next to the fastqrab binary")?;
    let mut cmd = Command::new(decompressor);
    cmd.arg("--format")
        .arg(format.as_arg())
        .arg("-P")
        .arg(thread_count.0.to_string())
        .arg(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    Ok(cmd)
}

/// On unix, enlarge the child's stdout pipe to 1 MiB (the usual
/// `/proc/sys/fs/pipe-max-size`). The default 64 KiB pipe forces the 4 MiB
/// decompressor chunks through in ~64 KiB reads, so producer and consumer
/// ping-pong on a near-empty/near-full buffer instead of overlapping (millions of
/// voluntary context switches). Best-effort: failure just keeps the default size.
/// No-op off unix.
fn enlarge_stdout_pipe(stdout: &std::process::ChildStdout) {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        const PIPE_BUF_SIZE: libc::c_int = 1024 * 1024;
        // SAFETY: `as_raw_fd` is a live pipe fd we own; F_SETPIPE_SZ takes an int.
        unsafe {
            libc::fcntl(stdout.as_raw_fd(), libc::F_SETPIPE_SZ, PIPE_BUF_SIZE);
        }
    }
    #[cfg(not(unix))]
    let _ = stdout;
}

/// Spawn our out-of-process `fastqrab-decompressor` to decode a gzip or zstd file
/// *by path*, returning a reader over its decoded stdout. Keeping the decoder in a
/// separate process isolates all of its `unsafe`/FFI from the main process.
/// Cross-platform: unix tunes the pipe size, other platforms read the child's
/// stdout directly.
pub fn spawn_decompressor(
    filename: &Path,
    format: DecompressorFormat,
    thread_count: ThreadCount,
) -> Result<DecompressorReader> {
    let mut cmd = decompressor_command(filename.as_os_str(), format, thread_count)?;
    let mut child = cmd.spawn().context(format!(
        "Failed to spawn fastqrab-decompressor process for file: {}.",
        filename.display()
    ))?; // cov:excl-line
    let stdout = child
        .stdout
        .take()
        .context("Failed to capture fastqrab-decompressor stdout")?;
    enlarge_stdout_pipe(&stdout);
    Ok(DecompressorReader {
        stdout,
        child,
        _feeder: None,
        exit_verified: false,
    })
}

/// As [`spawn_decompressor`] but for a path-less input (a declared handle, stdin):
/// run the child reading from its own stdin (`-`) and copy `input` into it on a
/// background thread. Cross-platform — the only contained way to decode bytes that
/// have no filesystem path.
pub fn spawn_decompressor_from_reader(
    mut input: Box<dyn Read + Send>,
    format: DecompressorFormat,
    thread_count: ThreadCount,
) -> Result<DecompressorReader> {
    let mut cmd = decompressor_command(std::ffi::OsStr::new("-"), format, thread_count)?;
    cmd.stdin(Stdio::piped());
    let mut child = cmd
        .spawn()
        .context("Failed to spawn fastqrab-decompressor process (stdin mode)")?;
    let mut stdin = child
        .stdin
        .take()
        .context("Failed to capture fastqrab-decompressor stdin")?;
    let stdout = child
        .stdout
        .take()
        .context("Failed to capture fastqrab-decompressor stdout")?;
    enlarge_stdout_pipe(&stdout);
    // Pump our input into the child. A broken pipe (child gone / early stop) just
    // ends the copy; it is not an error we can act on here.
    let feeder = std::thread::spawn(move || {
        let _ = std::io::copy(&mut input, &mut stdin);
    });
    Ok(DecompressorReader {
        stdout,
        child,
        _feeder: Some(feeder),
        exit_verified: false,
    })
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
    DecompressorFormat::from_compression(sniff_compression(&magic))
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
