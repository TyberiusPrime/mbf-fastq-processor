use crate::transformations::prelude::Result;
use crate::transformations::prelude::*; // union_find_rs::prelude also exports a Result

use bstr::{BStr, BString};
use disjoint::DisjointSet;
use fastqrab_io::{CompressionFormat, FileFormat};
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::num::NonZeroUsize;
use std::sync::OnceLock;
use std::time::Instant;

type WriterHandle = Arc<Mutex<Option<ChunkedRecordWriter>>>;
type DataHandles = Arc<Mutex<DemultiplexedData<Option<ChunkedRecordWriter>>>>;

fn encode_umi(umi: &[u8]) -> u32 {
    let mut v = 0u32;
    for &b in umi.iter().take(16) {
        v = (v << 2)
            | match b.to_ascii_uppercase() {
                b'A' => 0,
                b'C' => 1,
                b'G' => 2,
                b'T' => 3,
                _ => return u32::MAX, // any N -> becomes TTTTT
                                      // (which if you have less than 16 bp is distinguishable downstream
                                      // and if you don't you loose only a single umi
            };
    }
    v
}

fn human_fmt_usize(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('_');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn write_binary(
    matrix: IndexMap<(u32, u32), u32>,
    n_barcodes: u32,
    n_cells: u32,
    writer: &mut ChunkedRecordWriter,
) -> Result<()> {
    writer.write_text_record(b"%%MatrixMarket matrix coordinate integer general\n")?;
    writer.write_text_record(b"%metadata_json: {\"software\": \"fastqrab\"}\n")?;
    let total: usize = matrix.len();
    writer.write_text_record(format!("{n_barcodes} {n_cells} {total}\n").as_bytes())?;
    for ((gene, cell), count) in matrix {
        writer.write_text_record(format!("{} {} {}\n", gene, cell, count).as_bytes())?;
    }
    Ok(())
}

fn take_singleton_writer(output_files: &mut StepOutputFiles, id: &str) -> WriterHandle {
    let writers = output_files.take(id);
    let writer = writers
        .into_iter()
        .next()
        .map(|(_tag, w)| w)
        .expect("singleton writer must exist");
    Arc::new(Mutex::new(Some(writer)))
}

fn finish_writer(handle: &WriterHandle) -> Result<()> {
    if let Some(writer) = handle.lock().expect("lock poisoned").take() {
        let _summary = writer.finish()?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum LookupMode {
    Barcode,
    Label,
}

#[derive(Debug, JsonSchema, Copy, Clone)]
#[tpd]
enum UMIAggregation {
    None,
    Exact,
    #[tpd(alias = "1MM")]
    Cluster,
}
/// Collect (gene_idx, cell_idx, umi_2bit) triples per read, then write binary
/// data file(s) and lookup tables on finalize.
///
/// Both cell and gene barcodes are looked up by sequence in [barcodes.*] tables.
/// Index 0 is reserved for "unmatched"; real barcodes are 1-indexed.
/// Output: one binary file per demultiplex group + shared lookup .txt files.
#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct StoreSingleCellMatrix {
    /// Tag carrying the cell-barcode sequence (Location or String)
    cell_tag: TagLabel,

    /// Tag carrying the gene-barcode sequence (Location or String)
    gene_tag: TagLabel,

    /// Tag carrying the raw UMI sequence (Location or String)
    umi_tag: TagLabel,

    // how to aggregate UMIs into the matrix
    umi_aggregation: UMIAggregation,

    /// [barcode.*] sec listing all valid cell barcodes (sequence → name)
    cell_barcodes: TagLabel,

    /// [barcode.*] section listing all valid gene identifiers (sequence → name)
    gene_barcodes: TagLabel,

    /// Whether cell_tag values are barcode sequences (true) or corrected labels (false).
    /// Default: auto-detect — true for Location tags, false for String tags.
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "only read in get_tag_usage via PartialStoreSingleCellMatrix"
    )]
    cell_tag_contains_barcode: Option<bool>,
    ///
    /// Whether gene_tag values are barcode sequences (true) or corrected labels (false).
    /// Default: auto-detect — true for Location tags, false for String tags.
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "only read in get_tag_usage via PartialStoreSingleCellMatrix"
    )]
    gene_tag_contains_barcode: Option<bool>,

    /// Infix for output filenames
    #[tpd(default)]
    #[expect(dead_code, reason = "only used in declare_output_files")]
    infix: String,

    /// Compression for the binary data file (lookup tables are always plain text)
    #[tpd(default)]
    #[expect(dead_code, reason = "only used in neclare_output_files")]
    compression: CompressionFormat,

    #[tpd(default)]
    #[expect(dead_code, reason = "only used in declare_output_files")]
    compression_level: Option<u8>,

    // ---- runtime-only fields ----
    #[tpd(skip, default)]
    #[schemars(skip)]
    cell_lookup: Arc<FxIndexMap<BString, String>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    gene_lookup: Arc<FxIndexMap<BString, String>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    entries: Arc<Mutex<DemultiplexedData<Vec<[u32; 3]>>>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    max_umi_len: Arc<Mutex<u8>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    data_writer: Option<DataHandles>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    stats_writer: Option<DataHandles>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    cell_barcodes_writer: Option<WriterHandle>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    genes_writer: Option<WriterHandle>,

    #[tpd(skip)]
    #[schemars(skip)]
    cell_lookup_mode: LookupMode,

    #[tpd(skip)]
    #[schemars(skip)]
    gene_lookup_mode: LookupMode,
}

impl VerifyIn<PartialConfig> for PartialStoreSingleCellMatrix {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        Ok(())
    }
}

fn determine_lookup_mode(
    upstream_label_type: Option<&TagValueType>,
    tag_contains_barcode: &TomlValue<Option<bool>>,
) -> Option<LookupMode> {
    if let Some(upstream_label_type) = upstream_label_type {
        match upstream_label_type {
            TagValueType::Location | TagValueType::String => {
                if let Some(Some(tag_contains_barcode)) = tag_contains_barcode.as_ref() {
                    //user has explicitly told us what the tag contains,
                    //barcodes or barcode-names
                    if *tag_contains_barcode {
                        Some(LookupMode::Barcode)
                    } else {
                        Some(LookupMode::Label)
                    }
                } else if matches!(upstream_label_type, TagValueType::Location) {
                    Some(LookupMode::Barcode)
                } else {
                    Some(LookupMode::Label)
                }
            }
            // cov:excl-start
            _ => {
                //will be complained about because of allowed tag modes
                None
            } // cov:excl-stop
        }
    } else {
        None
    }
}

impl TagUser for PartialTaggedVariant<PartialStoreSingleCellMatrix> {
    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        let Some(inner) = self.toml_value.value.as_ref() else {
            return vec![];
        };
        let infix = inner.infix.as_ref().cloned().unwrap_or_default();
        let compression = inner.compression.as_ref().copied().unwrap_or_default();
        let compression_level = inner
            .compression_level
            .as_ref()
            .and_then(|x| x.as_ref())
            .copied();
        let data_suffix = compression.apply_suffix("matrix.mtx");
        let data_sink = SinkConfig {
            compression,
            compression_level,
            compression_threads: Some(NonZeroUsize::new(1).expect("Can't fail")),
            hash_uncompressed: false,
            hash_compressed: false,
            simulated_failure: None,
        };
        let span = inner.infix.span();
        vec![
            OutputDeclaration {
                id: "data".to_string(),
                target: WriteTargetConfig::new(vec![infix.clone(), "scd".to_string()], data_suffix),
                sink_config: data_sink.clone(),
                format: FileFormat::Text,
                chunk_policy: ChunkPolicy::default(),
                bam_options: None,
                singleton: false,
                span: span.clone(),
            },
            OutputDeclaration {
                id: "stats".to_string(),
                target: WriteTargetConfig::new(
                    vec![infix.clone(), "scd".to_string()],
                    compression.apply_suffix("matrix.mtx.stats.txt"),
                ),
                sink_config: data_sink.clone(),
                format: FileFormat::Text,
                chunk_policy: ChunkPolicy::default(),
                bam_options: None,
                singleton: false,
                span: span.clone(),
            },
            OutputDeclaration {
                id: "cell_barcodes".to_string(),
                target: WriteTargetConfig::new(
                    vec![infix.clone(), "scd".to_string()],
                    compression.apply_suffix("barcodes.txt"),
                ),
                sink_config: data_sink.clone(),
                format: FileFormat::Text,
                chunk_policy: ChunkPolicy::default(),
                bam_options: None,
                singleton: true,
                span: span.clone(),
            },
            OutputDeclaration {
                id: "genes".to_string(),
                target: WriteTargetConfig::new(
                    vec![infix.clone(), "scd".to_string()],
                    compression.apply_suffix("features.txt"),
                ),
                sink_config: data_sink,
                format: FileFormat::Text,
                chunk_policy: ChunkPolicy::default(),
                bam_options: None,
                singleton: true,
                span,
            },
        ]
    }

    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        let inner = self.toml_value.value.as_mut()?;
        inner.cell_lookup_mode = determine_lookup_mode(
            tags_available
                .get(inner.cell_tag.as_ref().expect("parent was ok"))
                .map(|meta| &meta.tag_type),
            &inner.cell_tag_contains_barcode,
        );
        inner.gene_lookup_mode = determine_lookup_mode(
            tags_available
                .get(inner.gene_tag.as_ref().expect("parent was ok"))
                .map(|meta| &meta.tag_type),
            &inner.gene_tag_contains_barcode,
        );

        Some(TagUsageInfo {
            used_tags: vec![
                inner
                    .cell_tag
                    .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                inner
                    .gene_tag
                    .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                inner
                    .umi_tag
                    .to_used_tag(&[TagValueType::String, TagValueType::Location]),
            ],
            used_barcodes: [
                inner.cell_barcodes.as_ref().cloned(),
                inner.gene_barcodes.as_ref().cloned(),
            ]
            .into_iter()
            .flatten()
            .collect::<std::collections::HashSet<TagLabel>>(),
            must_see_all_tags: true,
            ..Default::default()
        })
    }
}

fn seq_to_idx(seq: &[u8], map: &FxIndexMap<BString, String>) -> u32 {
    map.get_index_of(BStr::new(seq))
        .map(|i| i as u32 + 1)
        .unwrap_or(0)
}

impl StoreSingleCellMatrix {
    fn create_lookup(
        barcodes: &IndexMap<BString, String>,
        mode: LookupMode,
    ) -> Arc<FxIndexMap<BString, String>> {
        match mode {
            LookupMode::Barcode => Arc::new(
                barcodes
                    .iter()
                    .map(|(x, y)| (x.to_owned(), y.to_owned()))
                    .collect(),
            ),
            LookupMode::Label => Arc::new(
                barcodes
                    .iter()
                    .map(|(_k, label)| (BString::new(label.as_bytes().to_vec()), label.clone()))
                    .collect(),
            ),
        }
    }
}

impl Step for StoreSingleCellMatrix {
    fn init(
        &mut self,
        input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let cell_bc = input_info
            .barcodes_data
            .get(&self.cell_barcodes)
            .expect("cell_barcodes section missing from InputInfo");
        let gene_bc = input_info
            .barcodes_data
            .get(&self.gene_barcodes)
            .expect("gene_barcodes section missing from InputInfo");

        // cov:excl-start
        if cell_bc.seq_to_name.len() >= u32::MAX as usize {
            anyhow::bail!(
                "Too many cell barcodes: {} (max {})",
                cell_bc.seq_to_name.len(),
                u32::MAX as usize - 1
            );
        }
        if gene_bc.seq_to_name.len() >= u32::MAX as usize {
            anyhow::bail!(
                "Too many gene barcodes: {} (max {})",
                gene_bc.seq_to_name.len(),
                u32::MAX as usize - 1
            );
        }
        // cov:excl-end
        self.gene_lookup = Self::create_lookup(&gene_bc.seq_to_name, self.gene_lookup_mode);
        self.cell_lookup = Self::create_lookup(&cell_bc.seq_to_name, self.cell_lookup_mode);

        let per_tag = output_files.take("data");
        let mut entries_map = DemultiplexedData::new();
        let mut data_map = DemultiplexedData::new();
        for (tag, writer) in per_tag {
            entries_map.insert(tag, Vec::new());
            data_map.insert(tag, Some(writer));
        }
        let mut stats_map = DemultiplexedData::new();
        for (tag, writer) in output_files.take("stats") {
            stats_map.insert(tag, Some(writer));
        }

        self.entries = Arc::new(Mutex::new(entries_map));
        self.data_writer = Some(Arc::new(Mutex::new(data_map)));
        self.stats_writer = Some(Arc::new(Mutex::new(stats_map)));

        self.cell_barcodes_writer = Some(take_singleton_writer(&mut output_files, "cell_barcodes"));
        self.genes_writer = Some(take_singleton_writer(&mut output_files, "genes"));

        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let cell_tags = block
            .tags
            .get(&self.cell_tag)
            .expect("cell_tag not in block");
        let gene_tags = block
            .tags
            .get(&self.gene_tag)
            .expect("gene_tag not in block");
        let umi_tags = block.tags.get(&self.umi_tag).expect("umi_tag not in block");

        let cell_map = &*self.cell_lookup;
        let gene_map = &*self.gene_lookup;
        let output_tags = block.output_tags.as_ref();

        //todo: this is accidentially single threaded...
        let mut expected_umi_len = *self.max_umi_len.lock().expect("lock poisoned");

        let mut local_entries: DemultiplexedData<Vec<[u32; 3]>> = DemultiplexedData::new();

        for (ii, ((cell_val, gene_val), umi_val)) in cell_tags
            .iter()
            .zip(gene_tags.iter())
            .zip(umi_tags.iter())
            .enumerate()
        {
            let cell_idx = match cell_val {
                TagValue::String(s) => seq_to_idx(s, cell_map),
                TagValue::Location(hits) => seq_to_idx(&hits.joined_sequence_cow(None), cell_map),
                _ => 0,
            };

            let gene_idx = match gene_val {
                TagValue::String(s) => seq_to_idx(s, gene_map),
                TagValue::Location(hits) => seq_to_idx(&hits.joined_sequence_cow(None), gene_map),
                _ => 0,
            };

            let (umi_enc, umi_len) = match umi_val {
                TagValue::String(s) => {
                    if s.len() > 16 {
                        anyhow::bail!("UMI is {}bp, maximum supported length is 16bp", s.len());
                    }
                    (encode_umi(s), s.len() as u8)
                }
                TagValue::Location(hits) => {
                    let seq = hits.joined_sequence(None);
                    if seq.len() > 16 {
                        anyhow::bail!("UMI is {}bp, maximum supported length is 16bp", seq.len());
                    }
                    (encode_umi(&seq), seq.len() as u8)
                }
                _ => (0, 0),
            };

            {
                if expected_umi_len == 0 && umi_len > 0 {
                    *self.max_umi_len.lock().expect("lock poisoned") = umi_len;
                    expected_umi_len = umi_len;
                } else if umi_len > 0 && expected_umi_len != umi_len {
                    anyhow::bail!(
                        "UMI lengths are not uniform: expected {}bp, got {}bp",
                        expected_umi_len,
                        umi_len
                    );
                }
            }

            let output_tag = output_tags.map_or(0, |x| x[ii]);
            match local_entries.entry(output_tag) {
                std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(vec![[gene_idx, cell_idx, umi_enc]]);
                }
                std::collections::btree_map::Entry::Occupied(occupied_entry) => {
                    occupied_entry
                        .into_mut()
                        .push([gene_idx, cell_idx, umi_enc]);
                }
            }
        }

        let mut entries = self.entries.lock().expect("lock poisoned");
        for (demultiplex_tag, local) in local_entries.into_iter() {
            entries
                .get_mut(&demultiplex_tag)
                .expect("Entries were created in init")
                .extend(local);
        }

        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        // Sort and write one binary file per demultiplex tag
        {
            let entries_map = {
                let mut locked = self.entries.lock().expect("lock poisoned");
                std::mem::take(&mut *locked)
            };

            let mut data_guard = self
                .data_writer
                .as_ref()
                .expect("data_writer set in init")
                .lock()
                .expect("lock poisoned");
            let mut stat_guard = self
                .stats_writer
                .as_ref()
                .expect("data_writer set in init")
                .lock()
                .expect("lock poisoned");

            let mut any_gene_matches = false;
            let mut any_cell_matches = false;
            for (tag, mut entries) in entries_map.into_iter() {
                let mut stats = Vec::new();
                if let Some(Some(writer)) = data_guard.get_mut(&tag) {
                    //entries.par_sort_by_key(|&[g, c, _]| (g, c)); it's not measurably faster.

                    // how many different gene/cell combinations are there.
                    //let combos: HashSet<_> = entries.iter().map(|e| (e[0], e[1])).collect();
                    // dbg!(
                    //     "Tag {}, total entries: {}, unique gene/cell combos: {}",
                    //     tag,
                    //     entries.len(),
                    //     combos.len()
                    // );
                    entries.par_sort_unstable(); //must also sort by umi to be reproducible in output
                    let total = entries.len();
                    stats.push(("Input reads", total));

                    let matrix = aggregate_to_matrix(
                        entries,
                        self.umi_aggregation,
                        *self.max_umi_len.lock().expect("lock poisoned") as u16,
                    );

                    let unique_genes: FxHashSet<_> =
                        matrix.keys().filter(|(gene, _)| *gene != 0).collect();
                    let unique_cells: FxHashSet<_> =
                        matrix.keys().filter(|(_, cell)| *cell != 0).collect();

                    let matrix_matched = matrix
                        .keys()
                        .filter(|(gene, cell)| *gene != 0 && *cell != 0)
                        .count();

                    any_gene_matches |= !unique_genes.is_empty();
                    any_cell_matches |= !unique_cells.is_empty();
                    stats.push(("Matrix size", matrix.len()));
                    stats.push(("Matrix matched", matrix_matched));
                    stats.push(("Observed genes", unique_genes.len()));
                    stats.push(("Observed cells", unique_cells.len()));
                    let reads_in_matrix = matrix.values().map(|x| *x as usize).sum();
                    let reads_in_matrix_matched = matrix
                        .iter()
                        .filter(|((gene, cell), _)| *gene != 0 && *cell != 0)
                        .map(|(_, count)| *count as usize)
                        .sum();
                    stats.push(("Reads in matrix", reads_in_matrix));
                    stats.push((
                        "Reads in matrix with matched gene and cell",
                        reads_in_matrix_matched,
                    ));
                    let reads_lost_due_to_umi_cluster = total - reads_in_matrix as usize;
                    stats.push((
                        "Reads lost due to UMI deduplication",
                        reads_lost_due_to_umi_cluster,
                    ));

                    write_binary(
                        matrix,
                        self.cell_lookup.len() as u32 + 1, //+1 for unmatched. Cast is safe, we
                        //checked this before
                        self.gene_lookup.len() as u32 + 1,
                        writer,
                    )?;
                }
                if let Some(Some(writer)) = stat_guard.get_mut(&tag) {
                    writer.write_text_record(b"Metric\tValue\n")?;

                    // Pre-format so we can measure the widest value.
                    let rows: Vec<(&str, String)> = stats
                        .iter()
                        .map(|(metric, value)| (*metric, human_fmt_usize(*value)))
                        .collect();

                    let col_width = rows.iter().map(|(_, v)| v.len()).max().unwrap_or(0);

                    for (metric, value) in &rows {
                        writer.write_text_record(
                            format!("{}\t{:>width$}\n", metric, value, width = col_width)
                                .as_bytes(),
                        )?;
                    }
                }
            }
            if !any_gene_matches {
                bail!(
                    "No reads matched the gene barcode/label. \
                    Check your barcodes and possibly set `gene_tag_contains_barcode` \
                    to either 'barcode' or 'label' if the auto-detection has failed you.
                    "
                );
            }
            if !any_cell_matches {
                bail!(
                    "No reads matched the cell barcode/label. \
                    Check your barcodes and possibly set `cell_tag_contains_barcode` \
                    to either 'barcode' or 'label' if the auto-detection has failed you.
                    "
                );
            }
        }
        for (_tag, writer) in self
            .data_writer
            .as_ref()
            .expect("data_writer set in init")
            .lock()
            .expect("lock poisoned")
            .iter_mut()
        {
            if let Some(w) = writer.take() {
                let _summary = w.finish()?;
            }
        }

        // Write cell barcode lookup table (line 0 = "unmatched", then ordered names)
        {
            let handle = self
                .cell_barcodes_writer
                .as_ref()
                .expect("cell_barcodes_writer set in init");
            let mut guard = handle.lock().expect("lock poisoned");
            let writer = guard.as_mut().expect("writer not yet finished");
            writer.write_text_record(b"unmatched\n")?;
            for name in self.cell_lookup.values() {
                let mut line = name.as_bytes().to_vec();
                line.push(b'\n');
                writer.write_text_record(&line)?;
            }
        }
        finish_writer(
            self.cell_barcodes_writer
                .as_ref()
                .expect("cell_barcodes_writer set in init"),
        )?;

        // Write gene lookup table
        {
            let handle = self
                .genes_writer
                .as_ref()
                .expect("genes_writer set in init");
            let mut guard = handle.lock().expect("lock poisoned");
            let writer = guard.as_mut().expect("writer not yet finished");
            writer.write_text_record(b"unmatched\n")?;
            for name in self.gene_lookup.values() {
                let mut line = name.as_bytes().to_vec();
                line.push(b'\n');
                writer.write_text_record(&line)?;
            }
        }
        finish_writer(
            self.genes_writer
                .as_ref()
                .expect("genes_writer set in init"),
        )?;

        Ok(None)
    }
}

fn aggregate_to_matrix(
    entries: Vec<[u32; 3]>,
    umi_aggregation: UMIAggregation,
    umi_length: u16,
) -> IndexMap<(u32, u32), u32> {
    if entries.is_empty() {
        return IndexMap::new();
    } else {
        match umi_aggregation {
            UMIAggregation::None => aggregate_to_matrix_none(entries),
            UMIAggregation::Exact => aggregate_to_matrix_exact(entries),
            UMIAggregation::Cluster => aggregate_to_matrix_cluster(entries, umi_length),
        }
    }
}

fn aggregate_to_matrix_none(entries: Vec<[u32; 3]>) -> IndexMap<(u32, u32), u32> {
    let mut matrix = IndexMap::new();
    let mut counter = 0u32; //doesn't matter though, overwritten in first loop.
    let mut last = None;
    for entry in &entries {
        let gene_id = entry[0];
        let cell_id = entry[1];
        //let umi = entry[2];
        let key = (gene_id, cell_id);
        match last {
            Some(last_key) if last_key == key => {
                //same gene & cell
                counter = counter.saturating_add(1);
            }
            Some(last_key) => {
                // different gene & cell -> push last
                matrix.insert(last_key, counter);
                last = Some(key);
                counter = 1;
            }
            None => {
                //first trip through the loop
                last = Some(key);
                counter = 1;
            }
        }
    }
    if let Some(last) = last {
        matrix.insert(last, counter);
    }
    matrix
}

fn aggregate_to_matrix_exact(entries: Vec<[u32; 3]>) -> IndexMap<(u32, u32), u32> {
    let mut matrix = IndexMap::new();
    let mut counter = 0u32;
    let mut last: Option<(u32, u32, u32)> = None;
    for entry in &entries {
        let gene_id = entry[0];
        let cell_id = entry[1];
        let umi = entry[2];
        if umi == u32::MAX {
            //invalid umi / any N
            continue;
        }
        //we again use that we're umi sorted
        let key = (gene_id, cell_id, umi);
        match last {
            Some(last_key) if last_key == key => {
                //same gene & cell & umi. don't count
            }
            Some(last_key) if last_key.0 == key.0 && last_key.1 == key.1 => {
                //same cell and gene
                counter = counter.saturating_add(1);
                last = Some(key);
            }
            Some(last_key) => {
                // different gene & cell -> push last
                matrix.insert((last_key.0, last_key.1), counter);
                last = Some(key);
                counter = 1;
            }
            None => {
                //first trip through the loop
                last = Some(key);
                counter = 1;
            }
        }
    }
    if let Some(last_key) = last {
        matrix.insert((last_key.0, last_key.1), counter);
    }
    matrix
}

#[inline]
fn hamming_bp_16(a: u32, b: u32) -> u32 {
    // XOR marks differing bits within each 2-bit base.
    let x = a ^ b;

    // Collapse each 2-bit lane to 1 bit:
    // 00 -> 0
    // 01,10,11 -> 1
    let y = (x | (x >> 1)) & 0x5555_5555;

    y.count_ones()
}

///Aggregate each disjoint subgraph of umis within one hamming distance from each other
///to
// fn aggregate_to_matrix_cluster(
//     entries: Vec<[u32; 3]>,
//     umi_length: u16,
// ) -> IndexMap<(u32, u32), u32> {
//     let mut matrix = IndexMap::new();
//
//     if entries.is_empty() {
//         return matrix;
//     }
//
//     let mut last_key = (entries[0][0], entries[0][1]);
//     let mut seen: Vec<u32> = Vec::new();
//
//     for e in &entries {
//         let key = (e[0], e[1]);
//
//         if key != last_key {
//             //seen.sort_unstable(); we assume sortedness!
//             seen.dedup();
//
//             if !seen.is_empty() {
//                 //happens when only read had an N
//                 matrix.insert(last_key, umi_cluster_count(&seen, umi_length));
//             }
//
//             seen.clear();
//             last_key = key;
//         }
//
//         if e[2] != u32::MAX {
//             seen.push(e[2]);
//         }
//     }
//
//     //seen.sort_unstable();
//     seen.dedup();
//     if !seen.is_empty() {
//         matrix.insert(last_key, umi_cluster_count(&seen, umi_length));
//     }
//
//     matrix
// }

fn aggregate_to_matrix_cluster(
    entries: Vec<[u32; 3]>,
    umi_length: u16,
) -> IndexMap<(u32, u32), u32> {
    if entries.is_empty() {
        return IndexMap::new();
    }
    // Find split indices where (gene, cell) key changes
    let splits: Vec<usize> = (1..entries.len())
        .filter(|&i| entries[i][0] != entries[i - 1][0] || entries[i][1] != entries[i - 1][1])
        .collect();
    // Build range pairs [start, end) for each group
    let mut starts = Vec::with_capacity(splits.len() + 1);
    starts.push(0);
    let mut ends = splits.clone();
    ends.push(entries.len());
    starts.extend(splits);
    // Process each (gene, cell) group in parallel
    let results: Vec<((u32, u32), u32)> = starts
        .into_par_iter()
        .zip(ends.into_par_iter())
        .filter_map(|(start, end)| {
            let key = (entries[start][0], entries[start][1]);
            //todo: we don't need to check all of them for u32::MAx,
            //just the last one.
            let mut seen: Vec<u32> = entries[start..end]
                .iter()
                .filter(|e| e[2] != u32::MAX)
                .map(|e| e[2])
                .collect();
            seen.dedup();
            if seen.is_empty() {
                None
            } else {
                let count = umi_cluster_count(&seen, umi_length);
                Some((key, count))
            }
        })
        .collect();
    let mut matrix = IndexMap::with_capacity(results.len());
    for (key, count) in results {
        matrix.insert(key, count);
    }
    matrix
}

pub fn umi_cluster_count(umis: &[u32], umi_length: u16) -> u32 {
    if umis.len() == 1 {
        return 1;
    }
    debug_assert!(umis.len() >= 2);

    let values = umis;
    let n = values.len();

    //these branches do the same thing
    //but they differ in their O(n) and
    //we hence benchmark them on premise
    //to decide which ones to use
    let mut uf = DisjointSet::with_len(n);
    if n <= pairwise_threshold() {
        pairwise_union(&values, &mut uf);
    } else {
        neighbor_union_hash(&values, &mut uf, umi_length);
    }

    let mut roots = FxHashSet::default();

    for i in 0..n {
        roots.insert(uf.root_of(i));
    }

    roots.len() as u32
}

#[inline]
fn pairwise_union(values: &[u32], uf: &mut DisjointSet) {
    for i in 0..values.len() {
        let x = values[i];

        for j in (i + 1)..values.len() {
            let y = values[j];

            // dist <= 1
            if hamming_bp_16(x, y) <= 1 {
                uf.join(i, j);
            }
        }
    }
}

#[inline]
fn neighbor_union_hash(values: &[u32], uf: &mut DisjointSet, umi_length: u16) {
    assert!(
        umi_length <= 16,
        "UMI length must be at most 16bp to use neighbor_union_hash"
    );
    let mut index = FxHashMap::default();

    for (i, &x) in values.iter().enumerate() {
        index.insert(x, i);
    }

    for (i, &x) in values.iter().enumerate() {
        // dist == 1 basepair neighbors
        for bp in 0..umi_length {
            let shift = bp * 2;
            let current = (x >> shift) & 0b11;

            for replacement in 0..4u32 {
                if replacement == current {
                    continue;
                }
                // Clear the 2 bits at this basepair, then set the replacement
                let y = (x & !(0b11 << shift)) | (replacement << shift);

                if let Some(&j) = index.get(&y) {
                    uf.join(i, j);
                }
            }
        }
    }
}

/// It is unclear where the crossover between the O(n^2) pairwise, and O(32n) neighbor based
/// approach is. So we benchmark on the real system and make a decision.
static PAIRWISE_THRESHOLD: OnceLock<usize> = OnceLock::new();

fn pairwise_threshold() -> usize {
    *PAIRWISE_THRESHOLD.get_or_init(calibrate_pairwise_threshold)
}

fn calibrate_pairwise_threshold() -> usize {
    let candidates = [
        32usize,
        64,
        128,
        256,
        512,
        1024,
        1024 + 512,
        2048,
        2048 + 1024,
        4096,
    ];

    for &n in &candidates {
        // structured but non-trivial data
        let data: Vec<u32> = (0..n as u32).map(|x| x ^ (x << 1) ^ (x >> 1)).collect();
        let hash_data: FxHashSet<u32> = data.iter().copied().collect();
        assert!(hash_data.len() == data.len()); // ensure no duplicates, which would break the
        // benchmark

        // ----------------------------
        // PAIRWISE + DSU
        // ----------------------------
        let t_pair = {
            let mut uf = DisjointSet::with_len(n);

            let start = Instant::now();

            pairwise_union(&data, &mut uf);

            std::hint::black_box(uf);

            start.elapsed()
        };

        // ----------------------------
        // NEIGHBOR + DSU
        // ----------------------------
        let t_neighbor = {
            let mut uf = DisjointSet::with_len(n);

            let start = Instant::now();

            neighbor_union_hash(&data, &mut uf, 16);

            std::hint::black_box(uf);

            start.elapsed()
        };

        if t_neighbor < t_pair {
            // dbg!(format!(
            //     "Final threshold: {n}, time for pairwise: {t_pair:?}, time for neighbor: {t_neighbor:?}"
            // ));
            return n;
        }
    }
    *candidates.last().expect("must have candidate")
}

#[test]
fn test_pairwise_neighbor_aggreement() {
    let values = [91, 93, 94]; //if you were mutating bitwise instead of bytewise, this will fail
    let n = values.len();
    let mut uf = DisjointSet::with_len(n);
    neighbor_union_hash(&values, &mut uf, 16);
    let mut roots = FxHashSet::default();
    for i in 0..n {
        roots.insert(uf.root_of(i));
    }
    let l_hash = roots.len();
    let mut uf = DisjointSet::with_len(n);
    pairwise_union(&values, &mut uf);
    let mut roots = FxHashSet::default();
    for i in 0..n {
        println!("{}, {}, {}", i, values[i], uf.root_of(i));
        roots.insert(uf.root_of(i));
    }
    let l_pairwise = roots.len();
    assert_eq!(
        l_hash, l_pairwise,
        "pairwise and neighbor approaches should give the same result"
    );
}
