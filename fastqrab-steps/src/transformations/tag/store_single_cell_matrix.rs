use std::collections::{HashSet, VecDeque};

use crate::transformations::prelude::*;
use bstr::{BStr, BString};
use fastqrab_io::{CompressionFormat, FileFormat};
use petgraph::graph::UnGraph;
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

fn write_binary(
    matrix: IndexMap<(u32, u32), u32>,
    n_barcodes: u32,
    n_cells: u32,
    writer: &mut ChunkedRecordWriter,
) -> Result<()> {
    writer.write_text_record(b"%%MatrixMarket matrix coordinate integer general\n")?;
    writer.write_text_record(b"%metadata_json: {\"software\": \"fastqrab\"}\n")?;
    let total: usize = matrix.values().map(|x| *x as usize).sum();
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

#[derive(Debug)]
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
    tag_contains_barcode: Option<bool>,

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
    gene_seq_to_name: Arc<FxIndexMap<BString, String>>,

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

fn seq_to_idx(seq: &[u8], map: &FxIndexMap<BString, String>) -> u32 {
    map.get_index_of(BStr::new(seq))
        .map(|i| i as u32 + 1)
        .unwrap_or(0)
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

        self.gene_seq_to_name = Arc::new(
            //todo: if we extend toml_pretty_deser to support
            //collect into arbitrary IndexMaps,  we can go back to shared data here with the
            //barcodes
            gene_bc
                .seq_to_name
                .iter()
                .map(|(x, y)| (x.to_owned(), y.to_owned()))
                .collect(),
        );

        match self.lookup_mode {
            LookupMode::Barcode => {
                self.cell_lookup = Arc::new(
                    cell_bc
                        .seq_to_name
                        .iter()
                        .map(|(x, y)| (x.to_owned(), y.to_owned()))
                        .collect(),
                )
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

            for (tag, mut entries) in entries_map.into_iter() {
                if let Some(Some(writer)) = data_guard.get_mut(&tag) {
                    //entries.par_sort_unstable(); //must also sort by umi to be reproducible in output
                    entries.par_sort_by_key(|&[g, c, _]| (g, c));
                    let matrix = aggregate_to_matrix(entries, self.umi_aggregation);
                    write_binary(
                        matrix,
                        self.cell_lookup.len() as u32 + 1, //+1 for unmatched. Cast is safe, we
                        //checked this before
                        self.gene_seq_to_name.len() as u32 + 1,
                        writer,
                    )?;
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

fn aggregate_to_matrix(
    entries: Vec<[u32; 3]>,
    umi_aggregation: UMIAggregation,
) -> IndexMap<(u32, u32), u32> {
    match umi_aggregation {
        UMIAggregation::None => aggregate_to_matrix_none(entries),
        UMIAggregation::Exact => aggregate_to_matrix_exact(entries),
        UMIAggregation::Cluster => aggregate_to_matrix_cluster(entries),
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
    let mut last = None;
    let mut seen = std::collections::HashSet::new(); //todo: profile what to use here?
    for entry in &entries {
        let gene_id = entry[0];
        let cell_id = entry[1];
        let umi = entry[2];
        let key = (gene_id, cell_id);
        match last {
            Some(last_key) if last_key == key => {
                //same gene & cell
                if seen.insert(umi) {
                    counter = counter.saturating_add(1);
                }
            }
            Some(last_key) => {
                // different gene & cell -> push last
                matrix.insert(last_key, counter);
                last = Some(key);
                counter = 1;
                seen.clear();
                seen.insert(umi);
            }
            None => {
                //first trip through the loop
                last = Some(key);
                counter = 1;
                seen.insert(umi);
            }
        }
    }
    if let Some(last) = last {
        matrix.insert(last, counter);
    }
    dbg!(&matrix);
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
fn aggregate_to_matrix_cluster(entries: Vec<[u32; 3]>) -> IndexMap<(u32, u32), u32> {
    if entries.len() < 2 {
        return [((entries[0][0], entries[0][1]), 1)].into_iter().collect();
    }

    let mut matrix = IndexMap::new();
    let mut last = None;
    let mut seen = std::collections::HashSet::new(); //todo: profile what to use here?
    for entry in &entries {
        let gene_id = entry[0];
        let cell_id = entry[1];
        let umi = entry[2];
        let key = (gene_id, cell_id);
        match last {
            Some(last_key) if last_key == key => {
                seen.insert(umi);
            }
            Some(last_key) => {
                // different gene & cell -> push last
                matrix.insert(last_key, umi_cluster_count(&seen));
                last = Some(key);
                seen.clear();
                seen.insert(umi);
            }
            None => {
                //first trip through the loop
                last = Some(key);
                seen.insert(umi);
            }
        }
    }
    if let Some(last) = last {
        matrix.insert(last, umi_cluster_count(&seen));
    }
    dbg!(&matrix);
    matrix
}
///
/// Returns a vector of components, each as a Vec<N> of node values.
pub fn connected_components_values_undirected<E>(graph: &UnGraph<u32, E>) -> Vec<Vec<u32>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();

    for start in graph.node_indices() {
        if visited.contains(&start) {
            continue;
        }

        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            component.push(graph[node].clone());
            for neighbor in graph.neighbors(node) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        components.push(component);
    }

    components
}

fn umi_cluster_count(umis: &std::collections::HashSet<u32>) -> u32 {
    use std::collections::HashMap;
    let max_hamming = 1;
    let mut graph = UnGraph::<u32, ()>::new_undirected(); //todo: do not store edge
    //weights...
    let mut nodes = HashMap::new();
    for umi in umis.iter() {
        let node_index = graph.add_node(*umi);
        nodes.insert(umi, node_index);
    }
    let mut any_connected = false;
    for (umi1, umi2) in umis.iter().flat_map(|umi1| {
        umis.iter()
            .filter(move |umi2| umi1 < *umi2)
            .map(move |umi2| (umi1, umi2))
    }) {
        let dist = hamming_bp_16(*umi1, *umi2);
        if dist <= max_hamming {
            let node1 = nodes.get(umi1).expect("UMI should be in the graph");
            let node2 = nodes.get(umi2).expect("UMI should be in the graph");
            graph.add_edge(*node1, *node2, ());
            any_connected = true;
        }
    }
    if any_connected {
        let connected_components = connected_components_values_undirected(&graph);
        connected_components.len() as u32
    } else {
        umis.len() as u32
    }
}
