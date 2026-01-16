status: done
# Mutant analysis followup

src/config/deser.rs:267:5: replace option_btreemap_dna_string_from_string -> core::result::Result<Option<BTreeMap<BString, String>>, D::Error> with Ok(None)
src/io/reads.rs:640:26: replace == with != in WrappedFastQReadMut<'_>::is_empty
src/interactive.rs:31:24: replace % with / in get_local_time
src/config/deser.rs:97:13: replace string_or_seq_string_or_none::<impl de::Visitor<'de> for StringOrVec>::expecting -> fmt::Result with Ok(Default::default())
src/output.rs:1061:5: replace write_interleaved_blocks_to_bam -> Result<()> with Ok(())
src/interactive.rs:20:5: replace get_local_time -> String with String::new()
src/io/output.rs:38:26: replace + with * in write_read_to_bam
src/transformations/extract/low_quality_end.rs:60:36: replace - with + in <impl Step for LowQualityEnd>::apply
src/transformations/validation/spot_check_read_pairing.rs:109:30: replace += with *= in <impl Step for SpotCheckReadPairing>::apply
src/config/deser.rs:130:13: replace string_or_seq::<impl de::Visitor<'de> for StringOrVec>::expecting -> fmt::Result with Ok(Default::default())
src/interactive.rs:28:20: replace % with + in get_local_time
src/output.rs:116:21: replace match guard with true
src/io/reads.rs:885:30: replace + with - in WrappedFastQReadMut<'_>::trim_quality_start
src/interactive.rs:29:22: replace / with * in get_local_time
src/dna.rs:261:26: replace match guard with true
src/transformations/reports/progress.rs:12:36: replace / with % in format_seconds_to_hhmmss
src/io/reads.rs:1023:9: replace FastQBlocksCombined::apply_mut_with_tags with ()
src/main.rs:150:5: replace comment -> String with String::new()
src/io/reads.rs:726:43: replace - with / in WrappedFastQReadMut<'_>::trim_adapter_mismatch_tail
src/transformations/calc/complexity.rs:74:51: replace += with -= in <impl Step for Complexity>::apply
src/lib.rs:40:43: replace || with && in run
src/interactive.rs:276:5: replace modify_output_for_interactive -> Result<()> with Ok(())
src/io/output.rs:47:24: replace < with == in write_read_to_bam
src/transformations/reports/progress.rs:130:42: replace > with == in <impl Step for Progress>::apply
src/transformations/extract/poly_tail.rs:194:39: replace - with / in <impl Step for PolyTail>::apply
src/dna.rs:363:13: delete match arm
src/config.rs:597:17: replace | with ^ in Config::check_reports
src/config.rs:178:9: replace Config::check_name_collisions with ()
src/io/reads.rs:749:26: replace < with == in WrappedFastQReadMut<'_>::trim_poly_base_suffix::calc_run_length
src/transformations/tag.rs:185:35: replace > with < in validate_seed
src/demultiplex.rs:54:9: replace DemultiplexedData<T>::is_empty -> bool with true
src/transformations/reports/progress.rs:11:25: replace / with * in format_seconds_to_hhmmss
src/transformations/edits/reverse_complement_conditional.rs:69:64: replace + with * in <impl Step for ReverseComplementConditional>::apply
src/transformations/demultiplex.rs:44:9: replace <impl Step for Demultiplex>::validate_others -> Result<()> with Ok(())
src/io/reads.rs:719:24: replace > with < in WrappedFastQReadMut<'_>::trim_adapter_mismatch_tail
src/interactive.rs:369:5: replace display_success with ()
src/demultiplex.rs:201:9: replace OptDemultiplex::len -> usize with 1
src/transformations/reports/report_base_statistics_part2.rs:59:9: replace <impl Step for Box<_ReportBaseStatisticsPart2>>::transmits_premature_termination -> bool with true
src/pipeline.rs:56:45: delete ! in parse_interleaved_and_send
src/config/segments.rs:45:28: replace || with && in Segment::validate
src/transformations/extract/tag/other_file_by_name.rs:58:45: replace || with && in <impl Step for OtherFileByName>::validate_others
src/main.rs:332:21: replace > with == in canonicalize_variants
src/transformations/reports/progress.rs:12:28: replace % with + in format_seconds_to_hhmmss
src/transformations/extract/tag/other_file_by_name.rs:58:9: replace <impl Step for OtherFileByName>::validate_others -> Result<()> with Ok(())
src/io/reads.rs:897:9: replace WrappedFastQReadMut<'_>::trim_quality_end with ()
src/dna.rs:297:13: replace - with + in find_iupac_with_indel
src/output.rs:55:9: replace OutputRunMarker::preexisting -> bool with false
src/interactive.rs:393:5: replace display_error with ()
src/transformations/tag.rs:183:5: replace validate_seed -> Result<()> with Ok(())
src/main.rs:198:52: replace && with || in main
src/pipeline.rs:253:42: replace > with == in RunStage0::configure_demultiplex_and_init_stages
src/transformations/tag.rs:108:33: replace + with - in initial_filter_elements
src/config/deser.rs:341:13: replace u8_from_char_or_number::<impl serde::de::Visitor<'_> for Visitor>::visit_u8 -> Result<Self::Value, E> with Ok(Default::default())
src/transformations/tag/store_tag_in_fastq.rs:139:39: replace == with != in <impl Step for StoreTagInFastQ>::validate_others
src/dna.rs:323:5: replace contains_iupac_ambigous -> bool with true
src/io/reads.rs:640:9: replace WrappedFastQReadMut<'_>::is_empty -> bool with true
src/config/deser.rs:137:13: replace string_or_seq::<impl de::Visitor<'de> for StringOrVec>::visit_str -> Result<Self::Value, E> with Ok(Default::default())
src/io/reads.rs:900:19: replace < with == in WrappedFastQReadMut<'_>::trim_quality_end
src/transformations/edits/merge_reads.rs:326:60: replace * with + in find_best_overlap_fastp
src/transformations/extract/tag/other_file_by_sequence.rs:50:9: replace <impl Step for OtherFileBySequence>::validate_others -> Result<()> with Ok(())
src/config/deser.rs:404:20: replace match guard with false
src/transformations/reports/progress.rs:12:36: replace / with * in format_seconds_to_hhmmss
src/output.rs:864:25: replace > with < in write_text_block
src/transformations/tag/store_tag_in_fastq.rs:119:37: replace || with && in <impl Step for StoreTagInFastQ>::validate_others
src/config/input.rs:142:34: replace < with > in Input::init
src/transformations/extract/poly_tail.rs:172:47: replace < with == in <impl Step for PolyTail>::apply
src/io/reads.rs:884:19: replace < with == in WrappedFastQReadMut<'_>::trim_quality_start
src/transformations/extract/tag/duplicates.rs:189:13: delete match arm
src/io/reads.rs:909:36: replace - with / in WrappedFastQReadMut<'_>::trim_quality_end
src/io/output.rs:39:19: replace |= with ^= in write_read_to_bam
src/transformations/reports/report_count_oligos.rs:31:9: replace <impl Step for Box<_ReportCountOligos>>::transmits_premature_termination -> bool with true
src/transformations/calc/complexity.rs:74:51: replace += with *= in <impl Step for Complexity>::apply
src/transformations/tag/store_tags_in_table.rs:187:9: replace <impl Step for StoreTagsInTable>::finalize -> Result<Option<FinalizeReportResult>> with Ok(None)
src/dna.rs:67:9: replace TagValue::as_bool -> Option<bool> with Some(false)
src/transformations/edits/merge_reads.rs:282:24: replace < with == in find_best_overlap_fastp
src/cookbooks.rs:50:5: replace cookbook_count -> usize with 1
src/transformations/calc/complexity.rs:73:50: replace + with * in <impl Step for Complexity>::apply
src/interactive.rs:31:24: replace % with + in get_local_time
src/transformations/validation/all_reads_same_length.rs:82:25: delete match arm
src/io/reads.rs:513:9: replace <impl std::fmt::Debug for WrappedFastQReadMut<'_>>::fmt -> std::fmt::Result with Ok(Default::default())
src/transformations/calc/complexity.rs:77:54: replace - with / in <impl Step for Complexity>::apply
src/config.rs:319:36: replace % with + in Config::check_input_format_for_validation
src/transformations/tag/store_tags_in_table.rs:116:9: replace <impl Step for StoreTagsInTable>::transmits_premature_termination -> bool with false
src/io/output/compressed_output.rs:98:9: replace <impl Write for FailForTestWriter<T>>::flush -> io::Result<()> with Ok(())
src/transformations/edits/swap_conditional.rs:98:52: replace match guard with false
src/io/output.rs:36:19: replace |= with &= in write_read_to_bam
src/config.rs:568:17: replace || with && in Config::check_output
src/config/options.rs:65:5: replace default_output_buffer_size -> usize with 0
src/dna.rs:402:13: delete match arm
src/dna.rs:331:5: replace all_iupac -> bool with true
src/io/reads.rs:901:25: replace -= with += in WrappedFastQReadMut<'_>::trim_quality_end
src/config.rs:626:44: replace || with && in Config::check_for_any_output
src/transformations/extract/iupac_suffix.rs:104:39: replace - with / in <impl Step for IUPACSuffix>::apply
src/config/deser.rs:216:5: replace option_bstring_from_string -> core::result::Result<Option<BString>, D::Error> with Ok(Some(Default::default()))
src/transformations/reports/progress.rs:11:25: replace / with % in format_seconds_to_hhmmss
src/io/reads.rs:908:35: replace - with / in WrappedFastQReadMut<'_>::trim_quality_end
src/interactive.rs:196:12: delete ! in process_toml_interactive
src/transformations/edits/merge_reads.rs:330:55: replace < with > in find_best_overlap_fastp
src/demultiplex.rs:70:9: replace DemultiplexedData<T>::keys -> impl Iterator<Item = Tag>+'_ with ::std::iter::once(Default::default())
src/transformations/filters/head.rs:58:24: delete ! in <impl Step for Head>::apply
src/lib.rs:97:25: replace != with == in validate_config
src/config.rs:315:9: replace Config::check_input_format_for_validation with ()
src/cookbooks.rs:50:5: replace cookbook_count -> usize with 0
src/transformations/edits/swap_conditional.rs:98:58: replace == with != in <impl Step for SwapConditional>::apply
src/transformations/reports/report_duplicate_fragment_count.rs:33:9: replace <impl Step for Box<_ReportDuplicateFragmentCount>>::transmits_premature_termination -> bool with true
src/config/options.rs:65:10: replace * with / in default_output_buffer_size
src/transformations/extract/anchor.rs:71:9: replace <impl Step for Anchor>::validate_others -> anyhow::Result<()> with Ok(())
src/interactive.rs:247:17: delete match arm
src/interactive.rs:250:28: delete ! in make_paths_absolute
src/transformations/edits/reverse_complement_conditional.rs:69:64: replace + with - in <impl Step for ReverseComplementConditional>::apply
src/transformations/calc/complexity.rs:73:50: replace + with - in <impl Step for Complexity>::apply
src/dna.rs:403:13: delete match arm
src/io/output.rs:34:38: replace | with & in write_read_to_bam
src/transformations/calc/complexity.rs:71:34: replace > with == in <impl Step for Complexity>::apply
src/interactive.rs:242:29: replace || with && in make_paths_absolute
src/io/reads.rs:635:9: replace WrappedFastQReadMut<'_>::len -> usize with 1
src/config/deser.rs:267:5: replace option_btreemap_dna_string_from_string -> core::result::Result<Option<BTreeMap<BString, String>>, D::Error> with Ok(Some(BTreeMap::new()))
src/transformations/extract/longest_poly_x.rs:62:22: replace < with == in LongestPolyX::find_best_for_base
src/transformations/edits/reverse_complement_conditional.rs:69:46: replace - with / in <impl Step for ReverseComplementConditional>::apply
src/io/reads.rs:890:20: replace > with < in WrappedFastQReadMut<'_>::trim_quality_start
src/config/input.rs:214:43: replace == with != in Input::validate_stdin_usage
src/io/reads.rs:901:25: replace -= with /= in WrappedFastQReadMut<'_>::trim_quality_end
src/main.rs:520:72: replace == with != in find_single_valid_toml
src/config/deser.rs:91:5: replace string_or_seq_string_or_none -> Result<Option<Vec<String>>, D::Error> with Ok(Some(vec!["xyzzy".into()]))
src/transformations/reports/progress.rs:13:24: replace % with + in format_seconds_to_hhmmss
src/io/output.rs:38:26: replace + with - in write_read_to_bam
src/transformations/edits/merge_reads.rs:301:56: replace * with / in find_best_overlap_fastp
src/dna.rs:392:13: delete match arm
src/transformations/reports/inspect.rs:55:9: replace <impl std::fmt::Debug for Inspect>::fmt -> std::fmt::Result with Ok(Default::default())
src/config.rs:700:21: replace + with * in validate_barcode_disjointness
src/interactive.rs:309:5: replace inject_interactive_steps -> Result<()> with Ok(())
src/config/deser.rs:111:13: replace string_or_seq_string_or_none::<impl de::Visitor<'de> for StringOrVec>::visit_seq -> Result<Self::Value, S::Error> with Ok(Default::default())
src/main.rs:454:16: delete ! in validate_config_file
src/config.rs:159:29: replace == with != in Config::check_for_validation
src/output.rs:44:25: replace match guard with false
src/transformations/tag/store_tag_in_sequence.rs:92:58: replace < with > in <impl Step for StoreTagInSequence>::apply
src/pipeline.rs:593:33: replace <= with > in RunStage2::create_stage_threads
src/transformations/filters/head.rs:57:64: replace >= with < in <impl Step for Head>::apply
src/transformations/extract/longest_poly_x.rs:69:34: replace < with == in LongestPolyX::find_best_for_base
src/transformations/reports/progress.rs:57:9: replace <impl Step for Progress>::transmits_premature_termination -> bool with true
src/dna.rs:281:19: replace > with < in find_iupac_with_indel
src/cookbooks.rs:45:42: replace == with != in get_cookbook
src/io/reads.rs:907:20: replace < with == in WrappedFastQReadMut<'_>::trim_quality_end
src/interactive.rs:241:5: replace make_paths_absolute -> Result<()> with Ok(())
src/transformations/calc/complexity.rs:72:48: replace - with / in <impl Step for Complexity>::apply
src/transformations/extract/tag/other_file_by_name.rs:168:39: replace + with * in <impl Step for OtherFileByName>::apply
src/transformations/extract/regions_of_low_quality.rs:81:59: replace - with + in <impl Step for RegionsOfLowQuality>::apply
src/interactive.rs:64:5: replace run_interactive -> Result<()> with Ok(())
src/config/deser.rs:25:13: replace deserialize_map_of_string_or_seq_string::<impl de::Visitor<'de> for MapStringOrVec>::expecting -> fmt::Result with Ok(Default::default())
src/output.rs:44:25: replace match guard with true
src/transformations/validation/all_reads_same_length.rs:84:25: delete match arm
src/io/reads.rs:792:52: replace > with == in WrappedFastQReadMut<'_>::trim_poly_base_suffix::calc_run_length
src/interactive.rs:127:5: replace process_toml_interactive -> Result<String> with Ok(String::new())
src/transformations/extract/iupac_suffix.rs:90:37: replace > with == in <impl Step for IUPACSuffix>::apply
src/lib.rs:99:24: delete ! in validate_config
src/io/output/compressed_output.rs:82:24: replace < with > in <impl Write for FailForTestWriter<T>>::write
src/dna.rs:391:13: delete match arm
src/demultiplex.rs:58:9: replace DemultiplexedData<T>::iter -> impl Iterator<Item =(Tag, &T)> with ::std::iter::empty()
src/io/output.rs:34:15: replace |= with &= in write_read_to_bam
src/transformations/calc/kmers.rs:50:9: replace <impl Step for Kmers>::validate_others -> Result<()> with Ok(())
src/dna.rs:261:26: replace match guard with false
src/interactive.rs:30:33: replace / with * in get_local_time
src/io/parsers.rs:88:44: delete ! in <impl Parser for ChainedParser>::parse
src/transformations/edits/swap_conditional.rs:98:52: replace match guard with true
src/demultiplex.rs:137:9: replace <impl Clone for DemultiplexedData<T>>::clone -> Self with Default::default()
src/transformations/calc/kmers.rs:175:23: replace < with == in count_kmers_in_database
src/io/parsers/fastq.rs:106:21: delete match arm
src/io/output/compressed_output.rs:82:36: replace || with && in <impl Write for FailForTestWriter<T>>::write
src/dna.rs:67:9: replace TagValue::as_bool -> Option<bool> with None
src/io/reads.rs:505:9: replace <impl std::fmt::Debug for WrappedFastQRead<'_>>::fmt -> std::fmt::Result with Ok(Default::default())
src/transformations/extract/poly_tail.rs:38:22: replace < with == in PolyTail::calc_run_length
src/config.rs:319:40: replace == with != in Config::check_input_format_for_validation
src/interactive.rs:259:24: delete ! in make_paths_absolute
src/transformations/calc/expected_error.rs:81:50: replace += with *= in <impl Step for ExpectedError>::apply
src/transformations/edits/merge_reads.rs:380:23: replace >= with < in merge_at_offset_fastp::append_overlap
src/transformations/tag/store_tag_in_fastq.rs:177:20: delete ! in <impl Step for StoreTagInFastQ>::uses_tags
src/io/reads.rs:719:24: replace > with == in WrappedFastQReadMut<'_>::trim_adapter_mismatch_tail
src/transformations/extract/low_quality_end.rs:60:36: replace - with / in <impl Step for LowQualityEnd>::apply
src/config/deser.rs:377:13: replace opt_u8_from_char_or_number::<impl serde::de::Visitor<'de> for Visitor>::expecting -> std::fmt::Result with Ok(Default::default())
src/transformations/edits/merge_reads.rs:330:55: replace < with == in find_best_overlap_fastp
src/transformations/calc/complexity.rs:73:40: replace != with == in <impl Step for Complexity>::apply
src/transformations/calc/complexity.rs:77:41: replace += with -= in <impl Step for Complexity>::apply
src/interactive.rs:29:22: replace / with % in get_local_time
src/transformations/reports/report_base_statistics_part1.rs:97:30: replace += with -= in <impl Step for Box<_ReportBaseStatisticsPart1>>::apply::update_from_read
src/transformations/filters/by_numeric_tag.rs:25:9: replace <impl Step for ByNumericTag>::validate_others -> Result<()> with Ok(())
src/transformations/edits/rename.rs:65:9: replace <impl Step for Rename>::needs_serial -> bool with false
src/transformations/calc/complexity.rs:77:41: replace += with *= in <impl Step for Complexity>::apply
src/demultiplex.rs:74:9: replace DemultiplexedData<T>::values -> impl Iterator<Item =&T>+'_ with ::std::iter::empty()
src/main.rs:278:9: delete match arm
src/transformations/validation/all_reads_same_length.rs:62:45: replace += with *= in <impl Step for ValidateAllReadsSameLength>::apply
src/transformations/edits/swap.rs:58:48: replace match guard with true
src/io/reads.rs:881:9: replace WrappedFastQReadMut<'_>::trim_quality_start with ()
src/transformations/extract/longest_poly_x.rs:75:49: replace - with + in LongestPolyX::find_best_for_base
src/interactive.rs:225:5: replace modify_toml_for_interactive -> Result<()> with Ok(())
src/transformations/reports/progress.rs:126:30: replace % with / in <impl Step for Progress>::apply
src/main.rs:520:27: replace && with || in find_single_valid_toml
src/io/reads.rs:890:20: replace > with == in WrappedFastQReadMut<'_>::trim_quality_start
src/transformations/extract/tag/other_file_by_name.rs:139:43: replace + with * in <impl Step for OtherFileByName>::init
src/transformations/calc/expected_error.rs:81:50: replace += with -= in <impl Step for ExpectedError>::apply
src/main.rs:150:5: replace comment -> String with "xyzzy".into()
src/transformations/tag.rs:111:25: replace + with - in initial_filter_elements
src/transformations/edits/merge_reads.rs:279:54: replace + with * in find_best_overlap_fastp
src/transformations/reports/report_duplicate_count.rs:31:9: replace <impl Step for Box<_ReportDuplicateCount>>::transmits_premature_termination -> bool with true
src/config/deser.rs:91:5: replace string_or_seq_string_or_none -> Result<Option<Vec<String>>, D::Error> with Ok(Some(vec![]))
src/config/deser.rs:91:5: replace string_or_seq_string_or_none -> Result<Option<Vec<String>>, D::Error> with Ok(None)
src/interactive.rs:242:16: replace == with != in make_paths_absolute
src/io/output/compressed_output.rs:87:30: replace == with != in <impl Write for FailForTestWriter<T>>::write
src/config/deser.rs:104:13: replace string_or_seq_string_or_none::<impl de::Visitor<'de> for StringOrVec>::visit_str -> Result<Self::Value, E> with Ok(Default::default())
src/transformations/reports/progress.rs:13:24: replace % with / in format_seconds_to_hhmmss
src/transformations/tag/store_tag_in_fastq.rs:177:54: replace == with != in <impl Step for StoreTagInFastQ>::uses_tags
src/interactive.rs:283:32: replace == with != in modify_output_for_interactive
src/io/reads.rs:900:19: replace < with > in WrappedFastQReadMut<'_>::trim_quality_end
src/transformations/extract.rs:117:17: replace += with *= in extract_bool_tags_plus_all
src/interactive.rs:28:20: replace % with / in get_local_time
src/interactive.rs:257:17: delete match arm
src/io/output.rs:34:38: replace | with ^ in write_read_to_bam
src/interactive.rs:20:5: replace get_local_time -> String with "xyzzy".into()
src/transformations/reports/report_base_statistics_part1.rs:39:9: replace <impl Step for Box<_ReportBaseStatisticsPart1>>::transmits_premature_termination -> bool with true
src/interactive.rs:127:5: replace process_toml_interactive -> Result<String> with Ok("xyzzy".into())
src/transformations/edits/merge_reads.rs:380:36: replace && with || in merge_at_offset_fastp::append_overlap
src/transformations/extract/tag/other_file_by_sequence.rs:50:45: replace || with && in <impl Step for OtherFileBySequence>::validate_others
src/transformations/edits/swap_conditional.rs:99:58: replace == with != in <impl Step for SwapConditional>::apply
src/interactive.rs:193:12: delete ! in process_toml_interactive
src/config.rs:319:36: replace % with / in Config::check_input_format_for_validation
src/transformations/edits/merge_reads.rs:362:28: replace + with - in merge_at_offset_fastp::append_overlap
src/demultiplex.rs:54:9: replace DemultiplexedData<T>::is_empty -> bool with false
src/transformations.rs:193:9: replace Step::needs_serial -> bool with true
src/dna.rs:261:41: replace != with == in find_iupac_with_indel
src/transformations/calc/complexity.rs:72:48: replace - with + in <impl Step for Complexity>::apply
src/io/output/compressed_output.rs:82:47: replace < with > in <impl Write for FailForTestWriter<T>>::write
src/transformations/tag.rs:68:5: replace default_replacement_letter -> u8 with 0
src/io/reads.rs:796:50: replace += with *= in WrappedFastQReadMut<'_>::trim_poly_base_suffix::calc_run_length
src/transformations/edits/merge_reads.rs:303:51: replace < with > in find_best_overlap_fastp
src/config/options.rs:65:10: replace * with + in default_output_buffer_size
src/transformations/reports/progress.rs:129:40: replace / with * in <impl Step for Progress>::apply
src/io/output.rs:38:30: replace == with != in write_read_to_bam
src/transformations/calc/complexity.rs:62:53: replace - with / in <impl Step for Complexity>::apply
src/transformations/edits/merge_reads.rs:279:54: replace + with - in find_best_overlap_fastp
src/dna.rs:282:9: replace || with && in find_iupac_with_indel
src/transformations/extract/tag/duplicates.rs:86:41: replace > with < in <impl Step for Duplicates>::init
src/config.rs:216:45: replace != with == in Config::check_input_format
src/transformations/reports/report_base_statistics_part1.rs:96:30: replace += with -= in <impl Step for Box<_ReportBaseStatisticsPart1>>::apply::update_from_read
src/transformations/extract/longest_poly_x.rs:69:26: replace - with + in LongestPolyX::find_best_for_base
src/io/output.rs:33:22: replace > with < in write_read_to_bam
src/dna.rs:68:13: delete match arm
src/demultiplex.rs:22:9: replace <impl std::fmt::Debug for OutputWriter>::fmt -> std::fmt::Result with Ok(Default::default())
src/dna.rs:260:25: replace match guard with false
src/transformations.rs:146:9: replace Step::removes_tags -> Option<Vec<String>> with Some(vec![])
src/transformations/calc/complexity.rs:83:46: replace / with % in <impl Step for Complexity>::apply
src/io/reads.rs:884:19: replace < with > in WrappedFastQReadMut<'_>::trim_quality_start
src/transformations/demultiplex.rs:140:17: delete match arm
src/config/options.rs:65:5: replace default_output_buffer_size -> usize with 1
src/io/reads.rs:963:9: replace FastQBlocksCombined::is_empty -> bool with true
src/transformations/edits/swap_conditional.rs:99:52: replace match guard with false
src/transformations.rs:161:9: replace Step::uses_all_tags -> bool with true
src/config/deser.rs:216:5: replace option_bstring_from_string -> core::result::Result<Option<BString>, D::Error> with Ok(None)
src/interactive.rs:30:33: replace / with % in get_local_time
src/transformations/calc/complexity.rs:71:34: replace > with < in <impl Step for Complexity>::apply
src/interactive.rs:93:22: replace || with && in run_interactive
src/interactive.rs:96:16: delete ! in run_interactive
src/dna.rs:76:9: replace <impl From<f64> for TagValue>::from -> Self with Default::default()
src/demultiplex.rs:70:9: replace DemultiplexedData<T>::keys -> impl Iterator<Item = Tag>+'_ with ::std::iter::empty()
src/config/deser.rs:348:13: replace u8_from_char_or_number::<impl serde::de::Visitor<'_> for Visitor>::visit_i8 -> Result<Self::Value, E> with Ok(Default::default())
src/interactive.rs:197:16: delete ! in process_toml_interactive
src/demultiplex.rs:31:9: replace <impl std::fmt::Debug for DemultiplexedOutputFiles>::fmt -> std::fmt::Result with Ok(Default::default())
src/dna.rs:369:13: delete match arm
src/transformations/edits/merge_reads.rs:301:56: replace * with + in find_best_overlap_fastp
src/transformations/edits/convert_quality.rs:33:9: replace <impl Step for ConvertQuality>::validate_others -> Result<()> with Ok(())
src/io/reads.rs:908:35: replace - with + in WrappedFastQReadMut<'_>::trim_quality_end
src/transformations/extract/poly_tail.rs:80:48: replace > with == in PolyTail::calc_run_length
src/io/reads.rs:718:9: replace WrappedFastQReadMut<'_>::trim_adapter_mismatch_tail with ()
src/transformations/edits/merge_reads.rs:326:60: replace * with / in find_best_overlap_fastp
src/transformations/tag/store_tag_in_fastq.rs:122:40: replace || with && in <impl Step for StoreTagInFastQ>::validate_others
src/transformations/calc/complexity.rs:80:36: replace == with != in <impl Step for Complexity>::apply
src/interactive.rs:242:36: replace == with != in make_paths_absolute
src/interactive.rs:203:16: delete ! in process_toml_interactive
src/transformations/extract/poly_tail.rs:194:39: replace - with + in <impl Step for PolyTail>::apply
src/interactive.rs:202:12: delete ! in process_toml_interactive
src/transformations/reports/report_duplicate_fragment_count.rs:19:9: replace <impl Into<serde_json::Value> for DuplicateFragmentCountData>::into -> serde_json::Value with Default::default()
src/transformations/hamming_correct.rs:56:9: replace <impl Step for HammingCorrect>::validate_others -> Result<()> with Ok(())
src/io/reads.rs:907:20: replace < with > in WrappedFastQReadMut<'_>::trim_quality_end
src/transformations/edits/reverse_complement_conditional.rs:69:46: replace - with + in <impl Step for ReverseComplementConditional>::apply
src/transformations/reports/progress.rs:71:30: replace && with || in <impl Step for Progress>::validate_others
src/interactive.rs:30:25: replace % with / in get_local_time
src/transformations/validation/all_reads_same_length.rs:62:45: replace += with -= in <impl Step for ValidateAllReadsSameLength>::apply
src/dna.rs:39:9: replace TagValue::is_missing -> bool with true
src/io/reads.rs:909:36: replace - with + in WrappedFastQReadMut<'_>::trim_quality_end
src/transformations/tag.rs:68:5: replace default_replacement_letter -> u8 with 1
src/output.rs:1083:25: delete match arm
src/io/reads.rs:885:30: replace + with * in WrappedFastQReadMut<'_>::trim_quality_start
src/config/deser.rs:91:5: replace string_or_seq_string_or_none -> Result<Option<Vec<String>>, D::Error> with Ok(Some(vec![String::new()]))
src/transformations/tag/store_tag_in_sequence.rs:92:58: replace < with == in <impl Step for StoreTagInSequence>::apply
src/io/output.rs:34:15: replace |= with ^= in write_read_to_bam
src/output.rs:920:33: replace > with < in write_interleaved_text_block
src/dna.rs:39:9: replace TagValue::is_missing -> bool with false
src/transformations/convert/regions_to_length.rs:20:9: replace <impl Step for ConvertRegionsToLength>::validate_others -> Result<()> with Ok(())
src/config.rs:569:17: replace || with && in Config::check_output
src/interactive.rs:93:33: replace != with == in run_interactive
src/config/deser.rs:267:5: replace option_btreemap_dna_string_from_string -> core::result::Result<Option<BTreeMap<BString, String>>, D::Error> with Ok(Some(BTreeMap::from_iter([(Default::default(), String::new())])))
src/io/reads.rs:669:9: replace WrappedFastQReadMut<'_>::cut_end with ()
src/transformations/tag/store_tag_in_fastq.rs:104:9: replace <impl Step for StoreTagInFastQ>::validate_others -> Result<()> with Ok(())
src/config.rs:319:45: replace && with || in Config::check_input_format_for_validation
src/transformations/reports/progress.rs:129:40: replace / with % in <impl Step for Progress>::apply
src/transformations/tag/store_tag_in_fastq.rs:363:9: replace <impl Step for StoreTagInFastQ>::finalize -> Result<Option<crate::transformations::FinalizeReportResult>> with Ok(None)
src/interactive.rs:246:13: delete match arm
src/io/reads.rs:640:9: replace WrappedFastQReadMut<'_>::is_empty -> bool with false
src/io/reads.rs:635:9: replace WrappedFastQReadMut<'_>::len -> usize with 0
src/transformations.rs:156:9: replace Step::uses_tags -> Option<Vec<(String, &[TagValueType])>> with Some(vec![])
src/transformations/convert/eval_expression.rs:35:9: replace <impl std::fmt::Debug for EvalExpression>::fmt -> std::fmt::Result with Ok(Default::default())
src/config/deser.rs:384:13: replace opt_u8_from_char_or_number::<impl serde::de::Visitor<'de> for Visitor>::visit_none -> Result<Self::Value, E> with Ok(Default::default())
src/dna.rs:231:25: replace || with && in find_iupac_with_indel
src/transformations/extract/tag/duplicates.rs:57:9: replace <impl Step for Duplicates>::validate_others -> Result<()> with Ok(())
src/output.rs:44:36: replace == with != in OutputRunMarker::mark_complete
src/transformations/calc/kmers.rs:9:5: replace default_min_count -> usize with 0
src/transformations/validation/all_reads_same_length.rs:43:9: replace <impl Step for ValidateAllReadsSameLength>::needs_serial -> bool with false
src/dna.rs:67:9: replace TagValue::as_bool -> Option<bool> with Some(true)
src/config/input.rs:11:5: replace is_default -> bool with true
src/io/output.rs:35:26: replace == with != in write_read_to_bam
src/io/output.rs:39:19: replace |= with &= in write_read_to_bam
src/io/reads.rs:726:43: replace - with + in WrappedFastQReadMut<'_>::trim_adapter_mismatch_tail
src/transformations/reports/report_length_distribution.rs:25:9: replace <impl Step for Box<_ReportLengthDistribution>>::transmits_premature_termination -> bool with true
src/transformations/reports/progress.rs:12:28: replace % with / in format_seconds_to_hhmmss
src/transformations/edits/merge_reads.rs:380:42: replace <= with > in merge_at_offset_fastp::append_overlap
src/dna.rs:291:21: replace || with && in find_iupac_with_indel
src/config/deser.rs:267:5: replace option_btreemap_dna_string_from_string -> core::result::Result<Option<BTreeMap<BString, String>>, D::Error> with Ok(Some(BTreeMap::from_iter([(Default::default(), "xyzzy".into())])))
src/config/options.rs:61:9: replace * with + in default_buffer_size
src/transformations/edits/merge_reads.rs:314:28: replace < with == in find_best_overlap_fastp
src/transformations/calc/complexity.rs:83:46: replace / with * in <impl Step for Complexity>::apply
src/transformations/reports/report.rs:110:9: replace <impl Step for Report>::init -> Result<Option<DemultiplexBarcodes>> with Ok(None)
src/transformations/edits/swap_conditional.rs:99:52: replace match guard with true
src/config/deser.rs:320:13: replace u8_from_char_or_number::<impl serde::de::Visitor<'_> for Visitor>::expecting -> std::fmt::Result with Ok(Default::default())
src/config/segments.rs:135:28: replace || with && in SegmentSequenceOrName::validate
src/config/options.rs:57:5: replace default_thread_count -> usize with 1
src/interactive.rs:380:12: delete ! in display_success
src/transformations/extract/poly_tail.rs:172:47: replace < with > in <impl Step for PolyTail>::apply
src/config.rs:299:53: replace && with || in Config::check_input_format
src/transformations/extract/tag/duplicates.rs:100:53: replace / with * in <impl Step for Duplicates>::init
src/transformations/extract/tag/duplicates.rs:187:13: delete match arm
src/transformations/calc/complexity.rs:77:54: replace - with + in <impl Step for Complexity>::apply
src/main.rs:482:5: replace run_interactive_mode with ()
src/io/output.rs:36:19: replace |= with ^= in write_read_to_bam
src/transformations/validation/spot_check_read_pairing.rs:40:9: replace <impl Step for SpotCheckReadPairing>::validate_segments -> Result<()> with Ok(())
src/interactive.rs:30:25: replace % with + in get_local_time
