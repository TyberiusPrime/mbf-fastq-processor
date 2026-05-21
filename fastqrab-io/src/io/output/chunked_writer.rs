//! Record-oriented output abstraction for FASTQ / FASTA / BAM.
//!
//! # Layering (write direction, top → bottom)
//!
//! For FASTQ / FASTA:
//!
//! ```text
//!   plaintext bytes
//!     │
//!     ▼  HashLayer (uncompressed)         — optional sha256 of plaintext
//!     ▼  FailForTestLayer                 — pass-through unless tests inject failure
//!     ▼  CompressionLayer                 — Raw / Gzip / GzipParallel / Zstd
//!     ▼  HashLayer (compressed)           — optional sha256 of ciphertext
//!     ▼  BufWriter
//!     ▼  DataSink                         — File or Stdout
//! ```
//!
//! For BAM:
//!
//! ```text
//!   bam::Writer
//!     ▼  bgzf::MultithreadedWriter        — produces BGZF-framed bytes
//!     ▼  FailForTestLayer
//!     ▼  HashLayer (compressed)
//!     ▼  BufWriter
//!     ▼  DataSink
//! ```
//!
//! BAM has no uncompressed-hash layer because BGZF block boundaries are decided
//! inside noodles. Only the compressed-hash layer is meaningful.

#![allow(clippy::module_name_repetitions)]

use anyhow::{Context, Result};
use fastqrab_config::{CompressionFormat, FileFormat};
use sha2::Digest;
use std::io::{self, BufWriter, Write};
use std::num::{NonZero, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::simulated_failure::{FailForTestWriter, SimulatedWriteFailure};
use crate::ensure_output_destination_available;

// ---------------------------------------------------------------------------
// Layer 1: DataSink — file or stdout
// ---------------------------------------------------------------------------

/// The lowest level of the output stack: a real file or process stdout.
pub enum DataSink {
    File { path: PathBuf, file: ex::fs::File },
    Stdout(io::Stdout),
}

impl DataSink {
    pub fn create_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = ex::fs::File::create(&path)
            .with_context(|| format!("Could not open file for output: {}", path.display()))?;
        Ok(DataSink::File { path, file })
    }

    #[must_use]
    pub fn stdout() -> Self {
        DataSink::Stdout(io::stdout())
    }

    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            DataSink::File { path, .. } => Some(path),
            DataSink::Stdout(_) => None,
        }
    }
}

impl Write for DataSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            DataSink::File { file, .. } => file.write(buf),
            DataSink::Stdout(stdout) => stdout.lock().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            DataSink::File { file, .. } => file.flush(),
            DataSink::Stdout(stdout) => stdout.lock().flush(),
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 2: HashLayer
// ---------------------------------------------------------------------------

pub struct HashLayer<W: Write> {
    inner: W,
    hasher: Option<sha2::Sha256>,
}

impl<W: Write> HashLayer<W> {
    pub fn new(inner: W, enabled: bool) -> Self {
        Self {
            inner,
            hasher: if enabled {
                Some(sha2::Sha256::new())
            } else {
                None
            },
        }
    }

    /// Flush, return the inner writer and the hex sha256 (if enabled).
    pub fn finish(mut self) -> io::Result<(W, Option<String>)> {
        self.inner.flush()?;
        let hash = self.hasher.map(|h| format!("{:x}", h.finalize()));
        Ok((self.inner, hash))
    }
}

impl<W: Write> Write for HashLayer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        if let Some(hasher) = &mut self.hasher {
            hasher.update(&buf[..n]);
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// ---------------------------------------------------------------------------
// Layer 3: FailForTestLayer
// ---------------------------------------------------------------------------

/// Pass-through when no failure is configured, otherwise injects an error after
/// a budget of bytes has been written. Reuses [`FailForTestWriter`] from
/// `simulated_failure` so the test logic stays single-sourced.
enum FailForTestLayer<W: Write> {
    PassThrough(W),
    Active(FailForTestWriter<W>),
}

impl<W: Write> FailForTestLayer<W> {
    fn new(inner: W, failure: Option<SimulatedWriteFailure>) -> Self {
        match failure {
            None => FailForTestLayer::PassThrough(inner),
            Some(cfg) => FailForTestLayer::Active(FailForTestWriter::new(inner, cfg)),
        }
    }

    fn finish(self) -> W {
        match self {
            FailForTestLayer::PassThrough(w) => w,
            FailForTestLayer::Active(w) => w.finish(), //cov:excl-line we're not calling finish after a failure
        }
    }
}

impl<W: Write> Write for FailForTestLayer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            FailForTestLayer::PassThrough(w) => w.write(buf),
            FailForTestLayer::Active(w) => w.write(buf), //cov:excl-line we're not calling finish after a failure
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            FailForTestLayer::PassThrough(w) => w.flush(),
            FailForTestLayer::Active(w) => w.flush(),
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 4: CompressionLayer
// ---------------------------------------------------------------------------

pub enum CompressionLayer<W: Write + Send + 'static> {
    Raw(W),
    GzipParallel(ParallelGzipWriter<W>),
    Zstd(zstd::stream::Encoder<'static, W>),
}

/// Newtype around `gzp::par::compress::ParCompress` so callers don't need to
/// name `gzp` types.
pub struct ParallelGzipWriter<W: Write + Send + 'static> {
    inner: gzp::par::compress::ParCompress<'static, gzp::deflate::Gzip, W>,
}

impl<W: Write + Send + 'static> ParallelGzipWriter<W> {
    fn finish(mut self) -> Result<W> {
        use gzp::ZWriter;
        self.inner.finish().context("Parallel gzip finish failed")
    }
}

impl<W: Write + Send + 'static> Write for ParallelGzipWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write + Send + 'static> CompressionLayer<W> {
    pub fn new(
        inner: W,
        format: CompressionFormat,
        level: Option<u8>,
        threads: NonZeroUsize,
    ) -> Self {
        match format {
            CompressionFormat::Uncompressed => CompressionLayer::Raw(inner),
            CompressionFormat::Gzip => {
                let compression = match level {
                    Some(l) => flate2::Compression::new(u32::from(l).clamp(0, 9)),
                    None => flate2::Compression::default(),
                };
                let par = gzp::par::compress::ParCompressBuilder::<gzp::deflate::Gzip>::new()
                    .num_threads(threads.into())
                    .expect("only fails if t = 0, t never 0")
                    .compression_level(compression)
                    .from_writer(inner);
                CompressionLayer::GzipParallel(ParallelGzipWriter { inner: par })
            }
            CompressionFormat::Zstd => {
                let level = i32::from(level.unwrap_or(5)).clamp(1, 22);
                let enc = zstd::stream::Encoder::new(inner, level)
                    .expect("Failed to create zstd encoder - happens on invalid compression level, but we verified that");
                CompressionLayer::Zstd(enc)
            }
        }
    }

    pub fn finish(self) -> Result<W> {
        match self {
            CompressionLayer::Raw(w) => Ok(w),
            CompressionLayer::GzipParallel(p) => p.finish(),
            CompressionLayer::Zstd(e) => e.finish().context("Zstd finalisation failed"),
        }
    }
}

impl<W: Write + Send + 'static> Write for CompressionLayer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            CompressionLayer::Raw(w) => w.write(buf),
            CompressionLayer::GzipParallel(w) => w.write(buf),
            CompressionLayer::Zstd(w) => w.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            CompressionLayer::Raw(w) => w.flush(),
            CompressionLayer::GzipParallel(w) => w.flush(),
            CompressionLayer::Zstd(w) => w.flush(),
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 5: assembled per-format sinks
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct SinkConfig {
    pub compression: CompressionFormat,
    pub compression_level: Option<u8>,
    pub compression_threads: Option<NonZeroUsize>,
    pub hash_uncompressed: bool,
    pub hash_compressed: bool,
    pub simulated_failure: Option<SimulatedWriteFailure>,
}

impl SinkConfig {
    pub fn new_uncompressed_unhashed() -> Self {
        Self {
            compression: CompressionFormat::Uncompressed,
            compression_level: None,
            compression_threads: None,
            hash_uncompressed: false,
            hash_compressed: false,
            simulated_failure: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SinkHashes {
    pub uncompressed: Option<String>,
    pub compressed: Option<String>,
}

type TextStack = HashLayer<FailForTestLayer<CompressionLayer<HashLayer<BufWriter<DataSink>>>>>;

pub struct TextRecordSink {
    inner: TextStack,
}

// cov:excl-start
impl std::fmt::Debug for TextRecordSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextRecordSink").finish_non_exhaustive()
    }
}
// cov:excl-stop

impl TextRecordSink {
    pub fn new(sink: DataSink, config: &SinkConfig) -> Result<Self> {
        let buf = BufWriter::new(sink);
        let compressed_hash = HashLayer::new(buf, config.hash_compressed);
        let compress = CompressionLayer::new(
            compressed_hash,
            config.compression,
            config.compression_level,
            config
                .compression_threads
                .unwrap_or(NonZeroUsize::new(1).expect("can not fail")),
        );
        let fail = FailForTestLayer::new(compress, config.simulated_failure.clone());
        let plain_hash = HashLayer::new(fail, config.hash_uncompressed);
        Ok(TextRecordSink { inner: plain_hash })
    }

    pub fn finish(self) -> Result<(DataSink, SinkHashes)> {
        let (fail, uncompressed) = self.inner.finish()?;
        let compress = fail.finish();
        let compressed_hash_layer = compress.finish()?;
        let (buf, compressed) = compressed_hash_layer.finish()?;
        let sink = buf
            .into_inner()
            .map_err(|e| anyhow::anyhow!("Final flush failed: {}", e.error()))?;
        Ok((
            sink,
            SinkHashes {
                uncompressed,
                compressed,
            },
        ))
    }
}

impl Write for TextRecordSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Clone, Debug, Default)]
pub struct BamSinkOptions {
    pub comment_separation_char: u8,
    pub tag_to_bam_tags: Vec<([u8; 2], String)>,
    pub tag_to_reference: Option<String>,
    pub reference_sequences: Arc<Vec<(String, usize)>>,
    /// Pre-built and shared across all demultiplexed sinks so that the O(N)
    /// header construction happens exactly once, not once per output file.
    pub shared_header: Option<Arc<noodles::sam::Header>>,
}

impl BamSinkOptions {
    /// Build the SAM/BAM header from `reference_sequences` and cache it in
    /// `shared_header`. Call this once before cloning the options for each
    /// demultiplexed output so all sinks share the same Arc.
    pub fn build_shared_header(&mut self) {
        if self.shared_header.is_none() {
            self.shared_header = Some(Arc::new(create_bam_header(&self.reference_sequences)));
        }
    }
}

type BamStack = noodles::bgzf::io::multithreaded_writer::MultithreadedWriter<
    FailForTestLayer<HashLayer<BufWriter<DataSink>>>,
>;

pub struct BamRecordSink {
    writer: noodles::bam::io::Writer<BamStack>,
    header: Arc<noodles::sam::Header>,
    options: BamSinkOptions,
}

impl BamRecordSink {
    pub fn new(
        sink: DataSink,
        config: &SinkConfig,
        options: BamSinkOptions,
        thread_count: NonZero<usize>,
    ) -> Result<Self> {
        let buf = BufWriter::new(sink);
        let compressed_hash = HashLayer::new(buf, config.hash_compressed);
        let fail = FailForTestLayer::new(compressed_hash, config.simulated_failure.clone());

        let mut builder = noodles::bgzf::io::multithreaded_writer::Builder::default();
        if let Some(level) = config.compression_level {
            let level = noodles::bgzf::io::writer::CompressionLevel::try_from(level)
                .context("Invalid compression level for BAM BGZF writer")?;
            builder = builder.set_compression_level(level);
        }
        let bgzf = builder
            .set_worker_count(thread_count)
            .build_from_writer(fail);

        let mut writer = noodles::bam::io::Writer::from(bgzf);
        let header = options
            .shared_header
            .clone()
            .unwrap_or_else(|| Arc::new(create_bam_header(&options.reference_sequences)));
        writer
            .write_header(&header)
            .context("Failed to write BAM header")?;
        writer
            .get_mut()
            .flush()
            .context("Failed to flush BAM header to its own BGZF block")?;

        Ok(BamRecordSink {
            writer,
            header,
            options,
        })
    }

    pub fn write_record(
        &mut self,
        read: &crate::io::reads::WrappedFastQRead<'_>,
        read_index: usize,
        segment_index: usize,
        segment_count: usize,
        tags: &crate::io::reads::Tags,
    ) -> Result<()> {
        write_one_bam_record(
            &mut self.writer,
            &self.header,
            &self.options,
            read,
            read_index,
            segment_index,
            segment_count,
            tags,
        )
    }

    pub fn finish(self) -> Result<(DataSink, SinkHashes)> {
        let mut bgzf = self.writer.into_inner();
        let fail = bgzf
            .finish()
            .context("Failed to finish BAM multithreaded writer")?;
        let compressed_hash_layer = fail.finish();
        let (buf, compressed) = compressed_hash_layer.finish()?;
        let sink = buf
            .into_inner()
            .map_err(|e| anyhow::anyhow!("Final flush failed: {}", e.error()))?;
        Ok((
            sink,
            SinkHashes {
                uncompressed: None,
                compressed,
            },
        ))
    }
}

/// Same logic as `super::write_read_to_bam`, generic in the underlying writer
/// so it works against `BamRecordSink`'s stack.
#[expect(clippy::too_many_arguments, reason = "passthrough")]
#[expect(clippy::too_many_lines, reason = "ported as-is from write_read_to_bam")]
fn write_one_bam_record<W>(
    writer: &mut noodles::bam::io::Writer<W>,
    header: &noodles::sam::Header,
    options: &BamSinkOptions,
    read: &crate::io::reads::WrappedFastQRead<'_>,
    read_index: usize,
    segment_index: usize,
    segment_count: usize,
    tags: &crate::io::reads::Tags,
) -> Result<()>
where
    W: Write,
{
    use crate::io::reads::WrappedFastQReadCommon;
    use anyhow::bail;
    use bstr::{BStr, BString};
    use fastqrab_dna::dna::TagValue;
    use noodles::sam::alignment::{
        RecordBuf,
        io::Write as SamAlignmentWrite,
        record::{Flags as SamFlags, data::field::Tag},
        record_buf::{
            Data, QualityScores as SamQualityScores, Sequence as SamSequence, data::field::Value,
        },
    };

    let mut flags = SamFlags::UNMAPPED;
    if segment_count > 1 {
        flags |= SamFlags::SEGMENTED;
        flags |= SamFlags::MATE_UNMAPPED;
        if segment_index == 0 {
            flags |= SamFlags::FIRST_SEGMENT;
        }
        if segment_index + 1 == segment_count {
            flags |= SamFlags::LAST_SEGMENT;
        }
    }

    let adjusted_quality_scores = read
        .qual()
        .iter()
        .map(|&q| q.saturating_sub(33))
        .collect::<Vec<u8>>();
    let (name, comment) = if let Some(space_pos) = read
        .name()
        .iter()
        .position(|&c| c == options.comment_separation_char)
    {
        (
            &read.name()[..space_pos],
            Some(&read.name()[space_pos + 1..]),
        )
    } else {
        (read.name(), None)
    };

    let mut data_fields: Vec<(Tag, Value)> = Vec::new();
    if let Some(comment) = comment {
        data_fields.push((
            Tag::from([b'C', b'O']),
            Value::String(BString::from(comment)),
        ));
    }

    #[expect(clippy::cast_possible_truncation, reason = "BAM is f32")]
    for (bam_tag_bytes, fastqrab_tag_name) in &options.tag_to_bam_tags {
        let tag_values = tags
            .get(fastqrab_tag_name.as_str())
            .expect("Tag was missing? Config failure");
        let tag_value = &tag_values[read_index];

        let value_opt: Option<Value> = match tag_value {
            TagValue::String(s) => Some(Value::String(s.clone())),
            TagValue::Location(hits) => {
                let joined = hits
                    .0
                    .iter()
                    .map(|h| h.sequence.as_ref())
                    .collect::<Vec<_>>()
                    .join(b",".as_ref());
                Some(Value::String(BString::from(joined)))
            }
            TagValue::Numeric(n) => Some(Value::Float(*n as f32)),
            TagValue::Bool(b) => Some(Value::UInt8(u8::from(*b))),
            TagValue::Missing => None,
        };
        if let Some(value) = value_opt {
            data_fields.push((Tag::from(*bam_tag_bytes), value));
        }
    }

    let data: Data = data_fields.into_iter().collect();

    let mut reference_sequence_id: Option<usize> = None;
    if let Some(ref_tag_name) = &options.tag_to_reference
        && let Some(tag_values) = tags.get(ref_tag_name.as_str())
        && let Some(tag_value) = tag_values.get(read_index)
    {
        let ref_name = tag_value.to_bstr();
        let key: &[u8] = &ref_name;
        if !key.is_empty() {
            if let Some(idx) = header.reference_sequences().get_index_of(key) {
                reference_sequence_id = Some(idx);
                flags.remove(SamFlags::UNMAPPED);
            } else {
                bail!(
                    "Error in Bam tag-to-reference output: \n\
                        the value '{ref_name}' for tag '{ref_tag_name}' was not a valid reference sequence.\n\
                       Check that your output.bam.tag_to_reference.from_bam|from_barcodes derived values match\n\
                       with the actual tag values. Read name involved: '{}'",
                    BStr::new(read.name()),
                );
            }
        }
    }

    let mut record_builder = RecordBuf::builder()
        .set_name(name)
        .set_flags(flags)
        .set_sequence(SamSequence::from(read.seq().to_vec()))
        .set_quality_scores(SamQualityScores::from(adjusted_quality_scores))
        .set_data(data);

    if let Some(ref_id) = reference_sequence_id {
        record_builder = record_builder
            .set_reference_sequence_id(ref_id)
            .set_alignment_start(noodles_core::Position::MIN);
    }

    let record = record_builder.build();
    if let Err(e) = writer.write_alignment_record(header, &record) {
        let mut res = Err(e).context("Failed to write BAM record");
        let name_b: BString = name.into();
        if name_b.len() > 254 {
            res = res.context(format!(
                "The read name exceeded the 254 byte limited of the SAM/BAM spec.\n\
                    Shorten your read name, or set output.bam.comment_separation_char\n\
                    to split your read name into a name and a 'CO' tag (which may exceed 254 bytes).\n\
                    Read name (length: {len}): '{name_b}'",
                len = name_b.len()
            ));
        } else if name_b.is_empty() {
            res = res.context("Empty read name not supported by BAM. Check you Rename steps?");
        }
        if name_b
            .iter()
            .any(|&c| !(33..=126).contains(&c) || c == b'@')
        {
            res = res.context(format!(
                "The read name contains characters that are not allowed in the SAM/BAM spec.\n\
                    Remove or replace these characters, or set output.bam.comment_separation_char\n\
                    to split your read name into a name and a 'CO' tag (which may contain these characters).\n\
                    Read name: '{name_b}'"
            ));
        }
        return res;
    }
    Ok(())
}

fn create_bam_header(reference_sequences: &[(String, usize)]) -> noodles::sam::Header {
    use noodles::sam::header::record::value::{Map, map::ReferenceSequence};
    use std::str::FromStr;
    let base = "@HD\tVN:1.6\tSO:unsorted\n@PG\tID:fastqrab\tPN:fastqrab\n";
    let mut header = noodles::sam::Header::from_str(base).expect("static BAM header must parse");
    for (name, length) in reference_sequences {
        let rs = Map::<ReferenceSequence>::new(
            std::num::NonZero::try_from(*length)
                .expect("BAM reference sequence length must be > 0"),
        );
        header
            .reference_sequences_mut()
            .insert(bstr::BString::from(name.as_bytes()), rs);
    }
    header
}

// ---------------------------------------------------------------------------
// Layer 6: ChunkedRecordWriter
// ---------------------------------------------------------------------------

/// Private inner type for the [`WriteTargetConfig::File`] variant.
/// Fields are private to force construction through [`WriteTargetConfig::new`].
#[derive(Clone, Debug)]
pub struct FileTarget {
    infix_parts: Vec<String>,
    suffix: String,
}

impl FileTarget {
    pub fn infix_parts(&self) -> &[String] {
        &self.infix_parts
    }
    pub fn suffix(&self) -> &str {
        &self.suffix
    }
}

/// Directory- and prefix-free file target. The pipeline resolves this to a
/// full [`WriteTarget`] by prepending the output directory and prefix.
///
/// Construct via [`WriteTargetConfig::new`]; do not build variants directly.
#[derive(Clone, Debug)]
pub enum WriteTargetConfig {
    /// A file in the output directory. See [`FileTarget`] for field access.
    File(FileTarget),
    Stdout,
}

impl WriteTargetConfig {
    /// Construct a `WriteTargetConfig`. If `infix_parts == ["--stdout--"]`,
    /// returns [`Stdout`][Self::Stdout]; otherwise returns [`File`][Self::File].
    pub fn new(infix_parts: Vec<String>, suffix: String) -> Self {
        if infix_parts == ["--stdout--"] {
            Self::Stdout
        } else {
            Self::File(FileTarget {
                infix_parts,
                suffix,
            })
        }
    }
}

/// Everything a step needs to tell the pipeline about one output file it wants.
#[derive(Clone, Debug)]
pub struct OutputDeclaration {
    /// Step-local key used to retrieve the corresponding writer from
    /// [`StepOutputFiles`] inside [`Step::init`].
    pub id: String,
    pub target: WriteTargetConfig,
    pub sink_config: SinkConfig,
    pub format: FileFormat,
    pub chunk_policy: ChunkPolicy,
    pub bam_options: Option<BamSinkOptions>,
    /// If true, always produce a single writer (tag 0) regardless of demultiplexing.
    /// Use this for steps that write one aggregate output file (Inspect, Progress).
    pub singleton: bool,
    /// Byte range in the config source pointing at the field most responsible
    /// for this output (e.g. the `infix` or `in_label` field). Used by
    /// conflict-detection to produce precise error spans. Set to `0..0` when
    /// called from the runtime path (pipeline), where spans are not needed.
    pub span: std::ops::Range<usize>,
}

pub enum WriteTarget {
    Files(ChunkPaths),
    Stdout,
}

#[derive(Clone, Debug)]
pub struct ChunkPaths {
    pub directory: PathBuf,
    pub basename: String,
    pub suffix: String,
}

impl ChunkPaths {
    /// Path for chunk `index`. `digit_count` is `0` when chunking is disabled
    /// (meaning: no chunk-number infix).
    #[must_use]
    pub fn nth(&self, index: usize, digit_count: usize) -> PathBuf {
        let mut name = self.basename.clone();
        if digit_count > 0 {
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(&format!("{index:0digit_count$}"));
        }
        if !self.suffix.is_empty() {
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(&self.suffix);
        } //cov:excl-line
        self.directory.join(name)
    }

    /// Rename existing chunk files (and their hash sidecars) when the digit
    /// count grows, e.g. on the transition `9 -> 10` chunks.
    #[expect(clippy::string_slice, reason = "ascii filename arithmetic")]
    pub fn widen_existing(&self, old_digits: usize, new_digits: usize) -> Result<()> {
        let max_value = 10usize.pow(u32::try_from(old_digits).expect("digit count fits u32"));
        let mut existing: Vec<PathBuf> = Vec::new();
        for entry in ex::fs::read_dir(&self.directory).with_context(|| {
            format!(
                //cov:excl-start
                "Could not read output directory for renaming files: {}",
                self.directory.display()
            )
        })?
        //cov:excl-stop
        {
            existing.push(
                entry
                    .with_context(|| {
                        format!(
                            // cov:excl-start
                            "Could not read output directory entry for renaming files: {}",
                            self.directory.display()
                        )
                    })? //cov:excl-stop
                    .path(),
            );
        }
        for ii in 0..max_value {
            let old_prefix = self
                .directory
                .join(format!("{}.{ii:0old_digits$}", self.basename));
            let new_prefix = self
                .directory
                .join(format!("{}.{ii:0new_digits$}", self.basename));
            for path in &existing {
                if let Some(fname) = path.file_name().and_then(|s| s.to_str())
                    && fname.starts_with(
                        old_prefix
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .as_ref(),
                    )
                {
                    let suffix = &fname[old_prefix
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .len()..];
                    let new_name = new_prefix.with_file_name(format!(
                        "{}{}",
                        new_prefix.file_name().unwrap_or_default().to_string_lossy(),
                        suffix
                    ));
                    ex::fs::rename(path, &new_name).with_context(|| {
                        format!(
                            //cov:excl-start
                            "Could not rename output chunk file from {} to {}",
                            path.display(),
                            new_name.display()
                        )
                    })?; //cov:excl-stop
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkPolicy {
    pub records_per_chunk: Option<usize>,
}

impl ChunkPolicy {
    pub fn no_chunks() -> Self {
        ChunkPolicy {
            records_per_chunk: None,
        }
    }
}

// cov:excl-start
impl std::fmt::Debug for ChunkedRecordWriter {
    #[mutants::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkedRecordWriter")
            .field("format", &self.format)
            .field("chunk_index", &self.chunk_index)
            .field("records_in_chunk", &self.records_in_chunk)
            .finish_non_exhaustive()
    }
}
// cov:excl-stop

pub struct ChunkedRecordWriter {
    format: FileFormat,
    target: WriteTarget,
    sink_config: SinkConfig,
    chunk_policy: ChunkPolicy,
    bam_options: Option<BamSinkOptions>,
    bam_thread_count: NonZero<usize>,
    allow_overwrite: bool,

    /// Bytes written at the top of every chunk. Set via [`Self::set_header`].
    header: Option<Vec<u8>>,

    active: ActiveSink,
    chunk_index: usize,
    digit_count: usize,
    records_in_chunk: usize,
    completed_chunks: Vec<ChunkSummary>,
}

enum ActiveSink {
    Text(TextRecordSink),
    Bam(BamRecordSink),
    Idle,
}

impl ChunkedRecordWriter {
    /// Construct, validating writability and opening the first chunk.
    #[expect(
        clippy::too_many_arguments,
        reason = "they are all needed at construction time"
    )]
    pub fn new(
        format: FileFormat,
        target: WriteTarget,
        sink_config: SinkConfig,
        chunk_policy: ChunkPolicy,
        bam_options: Option<BamSinkOptions>,
        bam_thread_count: NonZero<usize>,
        allow_overwrite: bool,
    ) -> Result<Self> {
        match (&target, format, chunk_policy.records_per_chunk) {
            (WriteTarget::Stdout, FileFormat::Bam, _) => {
                anyhow::bail!("BAM output to stdout is not supported");
            }
            (WriteTarget::Stdout, _, Some(_)) => {
                anyhow::bail!("Chunked output is not supported when writing to stdout");
            }
            (WriteTarget::Stdout, FileFormat::None, _) => {
                anyhow::bail!("Cannot write 'none' format");
            }
            _ => {}
        }
        let digit_count = usize::from(chunk_policy.records_per_chunk.is_some());

        if let WriteTarget::Files(paths) = &target {
            let first_path = paths.nth(0, digit_count);
            let metadata = ensure_output_destination_available(&first_path, allow_overwrite)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                let is_fifo = metadata.as_ref().is_some_and(|m| m.file_type().is_fifo());
                if is_fifo && chunk_policy.records_per_chunk.is_some() {
                    anyhow::bail!(
                        "Chunked output is not supported when writing to named pipes: {}",
                        first_path.display()
                    );
                }
            }
            #[cfg(not(unix))]
            let _ = metadata;
        }

        let mut me = Self {
            format,
            target,
            sink_config,
            chunk_policy,
            bam_options,
            bam_thread_count,
            allow_overwrite,
            header: None,
            active: ActiveSink::Idle,
            chunk_index: 0,
            digit_count,
            records_in_chunk: 0,
            completed_chunks: Vec::new(),
        };
        me.open_active_sink()?;
        Ok(me)
    }

    /// Set bytes that are written at the start of every chunk (including the
    /// current one). Intended for CSV/TSV column headers and similar preambles.
    /// Not supported for BAM writers.
    pub fn set_header(&mut self, header: Vec<u8>) -> Result<()> {
        match &mut self.active {
            ActiveSink::Text(sink) => {
                sink.write_all(&header).context("Writing chunk header")?;
            }
            ActiveSink::Bam(_) => {
                anyhow::bail!("set_header is not supported for BAM ChunkedRecordWriter");
            }
            ActiveSink::Idle => unreachable!("active sink is Idle outside of rotation"),
        }
        self.header = Some(header);
        Ok(())
    }

    pub fn write_text_record(&mut self, encoded: &[u8]) -> Result<()> {
        match &mut self.active {
            ActiveSink::Text(sink) => sink
                .write_all(encoded)
                .context("Writing FASTQ/FASTA record")?,
            ActiveSink::Bam(_) => {
                anyhow::bail!("write_text_record called on a BAM ChunkedRecordWriter")
            }
            ActiveSink::Idle => unreachable!("active sink is Idle outside of rotation"),
        }
        self.records_in_chunk += 1;
        self.maybe_rotate()?;
        Ok(())
    }

    pub fn write_bam_record(
        &mut self,
        read: &crate::io::reads::WrappedFastQRead<'_>,
        read_index: usize,
        segment_index: usize,
        segment_count: usize,
        tags: &crate::io::reads::Tags,
    ) -> Result<()> {
        match &mut self.active {
            ActiveSink::Bam(sink) => {
                sink.write_record(read, read_index, segment_index, segment_count, tags)?;
            }
            ActiveSink::Text(_) => {
                anyhow::bail!("write_bam_record called on a text ChunkedRecordWriter")
            }
            ActiveSink::Idle => unreachable!("active sink is Idle outside of rotation"),
        }
        self.records_in_chunk += 1;
        self.maybe_rotate()?;
        Ok(())
    }

    #[must_use]
    pub fn format(&self) -> FileFormat {
        self.format
    }

    #[must_use]
    pub fn total_records_written(&self) -> u64 {
        let per_chunk = self.chunk_policy.records_per_chunk.unwrap_or(0);
        (self.chunk_index * per_chunk + self.records_in_chunk) as u64
    }

    fn maybe_rotate(&mut self) -> Result<()> {
        if let Some(limit) = self.chunk_policy.records_per_chunk
            && self.records_in_chunk >= limit
        {
            self.rotate_chunk()?;
        }
        Ok(())
    }

    fn rotate_chunk(&mut self) -> Result<()> {
        let old_index = self.chunk_index;
        let old_digits = self.digit_count;
        let old_active = std::mem::replace(&mut self.active, ActiveSink::Idle);
        self.finish_active_sink(old_active, old_index, old_digits)?;

        self.records_in_chunk = 0;
        self.chunk_index += 1;
        if self.chunk_index >= 10usize.pow(u32::try_from(self.digit_count).expect("u32")) {
            self.digit_count += 1;
            if let WriteTarget::Files(paths) = &self.target {
                paths.widen_existing(self.digit_count - 1, self.digit_count)?;
                // Update completed_chunks paths to reflect rename.
                for c in &mut self.completed_chunks {
                    if let Some(p) = c.path.take() {
                        let new_p = renamed_chunk_path(&p, self.digit_count - 1, self.digit_count);
                        c.path = Some(new_p);
                    }
                }
            }
        }
        self.open_active_sink()?;
        Ok(())
    }

    fn open_active_sink(&mut self) -> Result<()> {
        let sink = match &self.target {
            WriteTarget::Files(paths) => {
                let path = paths.nth(self.chunk_index, self.digit_count);
                if self.chunk_index > 0 {
                    let _ = ensure_output_destination_available(&path, self.allow_overwrite)?;
                }
                DataSink::create_file(path)?
            }
            WriteTarget::Stdout => DataSink::stdout(),
        };
        self.active = match self.format {
            FileFormat::Fastq | FileFormat::Fasta | FileFormat::Text => {
                ActiveSink::Text(TextRecordSink::new(sink, &self.sink_config)?)
            }
            FileFormat::Bam => {
                let opts = self.bam_options.clone().ok_or_else(|| {
                    anyhow::anyhow!("BAM options missing for BAM ChunkedRecordWriter")
                })?;
                ActiveSink::Bam(BamRecordSink::new(
                    sink,
                    &self.sink_config,
                    opts,
                    self.bam_thread_count,
                )?)
            }
            FileFormat::None => unreachable!("Cannot open ChunkedRecordWriter with format None"),
        };
        // Re-emit the header at the top of every chunk after the first (the
        // first chunk's header is written by set_header() itself).
        if self.chunk_index > 0 {
            if let (Some(header), ActiveSink::Text(sink)) = (&self.header, &mut self.active) {
                sink.write_all(header)
                    .context("Writing chunk header on rotation")?;
            }
        }
        Ok(())
    }

    fn finish_active_sink(
        &mut self,
        sink: ActiveSink,
        chunk_index: usize,
        digit_count: usize,
    ) -> Result<()> {
        let (data_sink, hashes) = match sink {
            ActiveSink::Text(mut s) => {
                // Extra explicit flush before finish to match the legacy main-pipeline
                // gzip output byte layout (one additional deflate sync block).
                s.flush()?;
                s.finish()?
            }
            ActiveSink::Bam(s) => s.finish()?,
            ActiveSink::Idle => unreachable!("finish_active_sink called on Idle"),
        };
        let path = data_sink.path().map(Path::to_path_buf);
        if let Some(path) = &path {
            if let Some(hash) = &hashes.uncompressed {
                write_hash_sidecar(path, hash, ".uncompressed.sha256")?;
            }
            if let Some(hash) = &hashes.compressed {
                write_hash_sidecar(path, hash, ".compressed.sha256")?;
            }
        }
        let _ = (chunk_index, digit_count);
        self.completed_chunks.push(ChunkSummary { path, hashes });
        Ok(())
    }

    pub fn finish(mut self) -> Result<ChunkedWriterSummary> {
        let total = self.total_records_written();
        let active = std::mem::replace(&mut self.active, ActiveSink::Idle);
        let idx = self.chunk_index;
        let digits = self.digit_count;
        self.finish_active_sink(active, idx, digits)?;
        Ok(ChunkedWriterSummary {
            records_written: total,
            chunks: self.completed_chunks,
        })
    }
}

#[must_use]
pub struct ChunkedWriterSummary {
    pub records_written: u64,
    pub chunks: Vec<ChunkSummary>,
}

#[derive(Debug)]
pub struct ChunkSummary {
    pub path: Option<PathBuf>,
    pub hashes: SinkHashes,
}

fn write_hash_sidecar(filename: &Path, hash: &str, suffix: &str) -> Result<()> {
    let hash_filename = filename.with_file_name(format!(
        "{}{}",
        filename.file_name().unwrap_or_default().to_string_lossy(),
        suffix
    ));
    let mut fh = ex::fs::File::create(&hash_filename)
        .with_context(|| format!("Could not open hash sidecar: {}", hash_filename.display()))?;
    fh.write_all(hash.as_bytes())?;
    fh.flush()?;
    Ok(())
}

#[expect(clippy::string_slice, reason = "ascii filename arithmetic")]
fn renamed_chunk_path(old: &Path, old_digits: usize, new_digits: usize) -> PathBuf {
    // Mirror what `widen_existing` does to a single recorded path.
    let Some(parent) = old.parent() else {
        return old.to_path_buf();
    };
    let Some(name) = old.file_name().and_then(|s| s.to_str()) else {
        return old.to_path_buf();
    };
    // Find the `.NNN[...]` segment after the last `.` group of the basename.
    // We re-derive by searching for an old-width digit run.
    // Simpler: scan dot-separated tokens for a numeric one of width `old_digits`.
    let mut parts: Vec<String> = name.split('.').map(ToString::to_string).collect();
    for part in &mut parts {
        if part.len() == old_digits && part.chars().all(|c| c.is_ascii_digit()) {
            let n: usize = part.parse().expect("digits");
            *part = format!("{n:0new_digits$}");
            break;
        }
    }
    parent.join(parts.join("."))
}
