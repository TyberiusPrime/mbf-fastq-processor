use crate::transformations::prelude::*;
use bstr::{BStr, BString};
use fastqrab_io::{CompressionFormat, FileFormat};
use rayon::prelude::*;

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
            };
    }
    v
}

/// Magic bytes + version: b"FQRSCD\x00\x02"
const BINARY_MAGIC: &[u8; 8] = b"FQRSCD\x00\x02";

fn write_binary(entries: &[[u32; 3]], umi_len: u8, writer: &mut ChunkedRecordWriter) -> Result<()> {
    writer.write_text_record(BINARY_MAGIC)?;
    writer.write_text_record(&(entries.len() as u32).to_le_bytes())?;
    writer.write_text_record(&[umi_len])?;
    for &[gene, cell, umi] in entries {
        let mut row = [0u8; 12];
        row[0..4].copy_from_slice(&gene.to_le_bytes());
        row[4..8].copy_from_slice(&cell.to_le_bytes());
        row[8..12].copy_from_slice(&umi.to_le_bytes());
        writer.write_text_record(&row)?;
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

#[derive(Debug)]
enum LookupMode {
    Barcode,
    Label,
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
pub struct StoreSingleCellData {
    /// Tag carrying the cell-barcode sequence (Location or String)
    cell_tag: TagLabel,

    /// Tag carrying the gene-barcode sequence (Location or String)
    gene_tag: TagLabel,

    /// Tag carrying the raw UMI sequence (Location or String)
    umi_tag: TagLabel,

    /// [barcode.*] section listing all valid cell barcodes (sequence → name)
    cell_barcodes: TagLabel,

    /// [barcode.*] section listing all valid gene identifiers (sequence → name)
    gene_barcodes: TagLabel,

    /// Whether cell_tag values are barcode sequences (true) or corrected labels (false).
    /// Default: auto-detect — true for Location tags, false for String tags.
    #[tpd(default)]
    #[expect(dead_code, reason = "only read in get_tag_usage via PartialStoreSingleCellData")]
    tag_contains_barcode: Option<bool>,

    /// Infix for output filenames
    #[tpd(default)]
    #[expect(dead_code, reason = "only used in declare_output_files")]
    infix: String,

    /// Compression for the binary data file (lookup tables are always plain text)
    #[tpd(default)]
    #[expect(dead_code, reason = "only used in declare_output_files")]
    compression: CompressionFormat,

    #[tpd(default)]
    #[expect(dead_code, reason = "only used in declare_output_files")]
    compression_level: Option<u8>,

    // ---- runtime-only fields ----
    #[tpd(skip, default)]
    #[schemars(skip)]
    cell_lookup: Arc<IndexMap<BString, String>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    gene_seq_to_name: Arc<IndexMap<BString, String>>,

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
    cell_barcodes_writer: Option<WriterHandle>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    genes_writer: Option<WriterHandle>,

    #[tpd(skip)]
    #[schemars(skip)]
    lookup_mode: LookupMode,
}

impl VerifyIn<PartialConfig> for PartialStoreSingleCellData {
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

impl TagUser for PartialTaggedVariant<PartialStoreSingleCellData> {
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
        let data_suffix = compression.apply_suffix("bin");
        let data_sink = SinkConfig {
            compression,
            compression_level,
            compression_threads: Some(1),
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
                id: "cell_barcodes".to_string(),
                target: WriteTargetConfig::new(
                    vec![infix.clone(), "scd".to_string()],
                    compression.apply_suffix("cell_barcodes.txt"),
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
                    compression.apply_suffix("genes.txt"),
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
        let upstream_label_type = tags_available
            .get(inner.cell_tag.as_ref().expect("parent was ok"))
            .map(|meta| &meta.tag_type);

        if let Some(upstream_label_type) = upstream_label_type {
            match upstream_label_type {
                TagValueType::Location | TagValueType::String => {
                    if let Some(Some(tag_contains_barcode)) = inner.tag_contains_barcode.as_ref() {
                        //user has explicitly told us what the tag contains,
                        //barcodes or barcode-names
                        if *tag_contains_barcode {
                            inner.lookup_mode = Some(LookupMode::Barcode);
                        } else {
                            inner.lookup_mode = Some(LookupMode::Label);
                        }
                    } else if matches!(upstream_label_type, TagValueType::Location) {
                        inner.lookup_mode = Some(LookupMode::Barcode);
                    } else {
                        inner.lookup_mode = Some(LookupMode::Label);
                    }
                }
                // cov:excl-start
                _ => {
                    //will be complained about because of allowed tag modes below
                } // cov:excl-stop
            }
        }

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

fn seq_to_idx(seq: &[u8], map: &IndexMap<BString, String>) -> u32 {
    map.get_index_of(BStr::new(seq))
        .map(|i| i as u32 + 1)
        .unwrap_or(0)
}

impl Step for StoreSingleCellData {
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

        self.gene_seq_to_name = gene_bc.seq_to_name.clone();

        match self.lookup_mode {
            LookupMode::Barcode => {
                self.cell_lookup = cell_bc.seq_to_name.clone();
            }
            LookupMode::Label => {
                self.cell_lookup = Arc::new(
                    cell_bc
                        .seq_to_name
                        .iter()
                        .map(|(_k, label)| (BString::new(label.as_bytes().to_vec()), label.clone()))
                        .collect(),
                );
            }
        }

        let per_tag = output_files.take("data");
        let mut entries_map = DemultiplexedData::new();
        let mut data_map = DemultiplexedData::new();
        for (tag, writer) in per_tag {
            entries_map.insert(tag, Vec::new());
            data_map.insert(tag, Some(writer));
        }
        self.entries = Arc::new(Mutex::new(entries_map));
        self.data_writer = Some(Arc::new(Mutex::new(data_map)));

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
        let gene_map = &*self.gene_seq_to_name;
        let output_tags = block.output_tags.as_ref();

        let mut entries = self.entries.lock().expect("lock poisoned");
        for (ii, ((cell_val, gene_val), umi_val)) in cell_tags
            .iter()
            .zip(gene_tags.iter())
            .zip(umi_tags.iter())
            .enumerate()
        {
            let cell_idx = match cell_val {
                TagValue::String(s) => seq_to_idx(s, cell_map),
                TagValue::Location(hits) => seq_to_idx(&hits.joined_sequence(None), cell_map),
                _ => 0,
            };

            let gene_idx = match gene_val {
                TagValue::String(s) => seq_to_idx(s, gene_map),
                TagValue::Location(hits) => seq_to_idx(&hits.joined_sequence(None), gene_map),
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
                let mut expected = self.max_umi_len.lock().expect("lock poisoned");
                if *expected == 0 && umi_len > 0 {
                    *expected = umi_len;
                } else if umi_len > 0 && *expected != umi_len {
                    anyhow::bail!(
                        "UMI lengths are not uniform: expected {}bp, got {}bp",
                        *expected,
                        umi_len
                    );
                }
            }

            let output_tag = output_tags.map_or(0, |x| x[ii]);
            if let Some(bucket) = entries.get_mut(&output_tag) {
                bucket.push([gene_idx, cell_idx, umi_enc]);
            }
        }

        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        // Sort and write one binary file per demultiplex tag
        {
            let mut entries_map = {
                let mut locked = self.entries.lock().expect("lock poisoned");
                std::mem::take(&mut *locked)
            };

            for (_tag, entries) in entries_map.iter_mut() {
                entries.par_sort_unstable();
            }

            let umi_len = *self.max_umi_len.lock().expect("lock poisoned");

            let mut data_guard = self
                .data_writer
                .as_ref()
                .expect("data_writer set in init")
                .lock()
                .expect("lock poisoned");

            for (tag, entries) in entries_map.iter_mut() {
                if let Some(Some(writer)) = data_guard.get_mut(&tag) {
                    write_binary(entries, umi_len, writer)?;
                }
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
            for name in self.gene_seq_to_name.values() {
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
