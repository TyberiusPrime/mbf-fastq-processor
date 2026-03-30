# Changelog


## v0.9.0

### General
- Renamed project from mbf-fastq-processor to fastqrab
- Much improved error messages pinpointing exactly what needs to change.


### New & renamed steps
- New step: ConcatTags — concatenate multiple tags into one
- New step: Lowercase — unified replacement for LowercaseTag/LowercaseSequence
- New step: SpotCheckReadPairing — Hamming-distance based read pairing validation
- New step: ExtractIUPACSuffix added
- OtherFile unified: OtherFileByName and OtherFileBySequence merged into one step
- ExtractAnchor merged into ExtractRegions

### Step changes

- Conditional Swap/ReverseComplement variants merged into the main steps
- [if tag]({{< relref "docs/concepts/tag.md#conditional-processing" >}}) support on 8 core editing steps for conditional read editing
- ExtractIUPAC: multiple queries in one step, max_mismatches now required, large performance improvements
- min_length added to ExtractRegionsOfLowQuality
- Quality checking added to Prefix/Postfix
- Tag replacement within regular expressions

### Output changes

- Tag histogram reports; demultiplex data nested under 'demultiplex' key in reports

### Performance

- Redesigned multi-core engine: workpool based, better controllable, better documented
- Default thread count now uses all available CPU cores
- Rapidgzip for parallel gzip decompression, now also for FASTA; auto-detected; included in Nix builds
- Arena-based parsers for FASTA and BAM
- Parallel BAM decoding
- Multicore EvalExpression, ReportCountOligos, ReportLengthDistribution
- Prefix/Postfix massively improved performance
- Merge base statistics ~80% faster
- ConcatTags ~15% faster
- IUPAC matching: replaced Sassy with optimized pure-Rust implementation
- Optimized SwapConditional, TrimAtTag, StoreTagInSequence, FilterReservoirSample, Rename
- Dynamic cuckoo filter sizing; initial_filter_capacity documented; read count estimation

### Other
- verify command: validates a pipeline produces expected output; auto-detects config, captures stdout/stderr
- configuration toml can now be read from stdin (incompatible with reads from stdin).
- Shell autocompletion for bash, fish, and zsh
- benchmark mode and per-step benchmark harness
- template command: shows help on error
- LLM configuration guide and template.toml rewrite for LLM-assisted config generation
- TagLabel type: all tag names are now strongly typed; duplicate tag names produce a clear error
- IndexMap replaces HashMap everywhere for deterministic output order
- unwrap() replaced with expect() throughout; clippy::unwrap_used now denied
- MSRV pinned to match flake.nix Rust version
- Security: upgraded bytes crate (GHSA-434x-w66g-qw3r)
- Upgraded dependencies

### Documentation
- Four new cookbooks for common FastQ processing tasks
- Copy-to-clipboard button in docs
- Documentation URLs included in validation failure messages
- Added mascot

### Bug fixes
- Fixed fastp merge algorithm (replaced with direct port of the reference C++ algorithm)
- Fixed invalid FASTQ detection when comment line doesn't start with '+'
- Fixed Windows newline detection edge case in parser
- Fixed Local-Local FastQElement swap
- Fixed demultiplex & fragment count in reports
- Fixed Head short-circuit (broken by SpotCheckReadPairing)
- Fixed ignore_unaligned → now include_mapped / include_unmapped
- Fixed barcode overlapping multiple matches



## v0.8.1
   
- Github release workflow test


## v0.8.0

- Versioned documentation
- First revision where very major feature is in place. Changelog starts here.

