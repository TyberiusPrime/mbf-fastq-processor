#![allow(clippy::identity_op)]
mod common;
use anyhow::{Context, Result};
use common::*;
use std::io::{Read, Write};
use std::path::Path;

fn read_compressed(filename: impl AsRef<Path>) -> Result<String> {
    let fh = std::fs::File::open(filename.as_ref())
        .with_context(|| format!("Could not open file {:?}", filename.as_ref()))?;
    let mut wrapped = niffler::send::get_reader(Box::new(fh))?;
    let mut out: Vec<u8> = Vec::new();
    wrapped.0.read_to_end(&mut out)?;
    Ok(std::str::from_utf8(&out)?.to_string())
}

#[test]
fn test_noop() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_noop_minimal() {
    //
    let td = run("
[input]
    read1 = 'sample_data/minimal.fq'
[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/minimal.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_validate_seq() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'ValidateSeq'
    allowed = 'CGATN'
    target = 'Read1'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_validate_phred() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'ValidatePhred'
    target = 'Read1'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_cat() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ten_reads.fq', 'sample_data/ten_reads.fq']

[options]
    accept_duplicate_files = true

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    let should = format!("{should}{should}");
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_skip() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[options]
    block_size = 2

[[step]]
    action='Skip'
    n = 5

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    //keep final 20 lines of should
    let mut should = should.lines().skip(20).collect::<Vec<_>>().join("\n");
    should.push('\n');
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_gz_input() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR664392_1250.fq.gz']

[options]
    block_size = 10 # to test that Head is actually total

[[step]]
    action='Head'
    n = 5

[output]
    prefix = 'temp'
");
    assert!(td.path().join("temp_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("temp_1.fq")).unwrap();
    let should = "@ERR664392.1 GAII02_0001:7:1:1116:18963#0/1
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@ERR664392.2 GAII02_0001:7:1:1116:17204#0/1
GGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
@ERR664392.3 GAII02_0001:7:1:1116:15799#0/1
GTGCACTGCTGCTTGTGGCTNTCCTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCC=@@B@#C>C?##########################
@ERR664392.4 GAII02_0001:7:1:1116:17486#0/1
GGAAGTTGATCTCATCCTGANGAGCATNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCC@CCCBCCCCCCC@?C#AAAA##########################
@ERR664392.5 GAII02_0001:7:1:1116:15631#0/1
TTCAAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNNNN
+
BCCCCCCCCCCCCCCCCCCC#ABBB##########################
";
    assert_eq!(actual.chars().filter(|x| *x == '\n').count(), 5 * 4);
    assert_eq!(should, actual);
}

fn test_860_head_5(td: &tempfile::TempDir, suffix: &str) {
    let actual = read_compressed(td.path().join(format!("output_1{suffix}"))).unwrap();
    let should = "@ERR12828869.1 A00627:18:HGV7TDSXX:3:1101:10004:10269/1
ATTGAGTACAAAAAACCTTACATAAATTAAAGAATGAATACATTTACAGGTGTCGATGCAAACGTTCCCAACTCAAGGCAACTAACAACCGATGGTGGTCAGGAGGGAAGAAACCAGAACTGAAACTGGGTCCTAAGGCTCGGACTTTCC
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFF,FF:FFFFF::FFFFFFFF:FFFFFFFF:FFFFFFFFFFFFFF,FFF:FFFFFFFFFFF,FFFFFFFFFFF,FFFFFFFFFFFFFFFFFFF:FFF,
@ERR12828869.2 A00627:18:HGV7TDSXX:3:1101:10004:13401/1
ACTATGTAAGGCTGTCGTTTTACATAGTTTTAATGAGGAAACGATTGCTTTCCACTTGTGATCTGAGCCACTGACATAGACTGTGCACAAATACTGTAGACATTCCTCTAGAGTCTGAGGTAGCATGGGTCAAAGGCCAACATGACAGTC
+
FFFFFFFFFFFFFFFFFF,FFFF:FFF:FFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFF:FFFFFF:FFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFF:FFF::FFFF:F:FF,FFFFFFFFFFFFFFFFFFFFFFFF,FF:F:FF
@ERR12828869.3 A00627:18:HGV7TDSXX:3:1101:10004:14998/1
CACCTTTCCCCTTCCTGTCACTCATGTGGACCTCATATAAGGGAAAGATACTCTCAACCTCTTGTATTTGGAGAGTTTTGAGCAGACAGGTAGAAGATGGAGCCTGGGAGCAGCTGTTTTTCCAATAGTCAAATTAGGACTGTTTCTCTC
+
FFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFF:FFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
@ERR12828869.4 A00627:18:HGV7TDSXX:3:1101:10004:1752/1
CGCAGAGGGCTGGTTCATTTCAGATCCTTCACTGCCAAACCCGGGGGTAGGGACTGCTTCAGCTTCTCTGCCTTTTCCTTGTCTGTGATAACCAGGGTGTAAAGGTACCTGCTGCAGCGAACCTTGAACTTCACATTATCCTTGTTCTTC
+
FFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFF,FF
@ERR12828869.5 A00627:18:HGV7TDSXX:3:1101:10004:17534/1
CTGGTGGTAGGCCCGACAGATGATGGCTGTTTCTTGGAGCTGAGGGTATGCAGCATCCAGCGCAACCGCTCTGCGTGTCGTGTTCTTCGAGCAGGTCAGGCTGCTACACTCGCCCTTGGAGACTTTGACCGTGCATTGCTTCGCAAGGGC
+
FFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
";
    compare_fastq(&actual, should);

    let actual = read_compressed(td.path().join(format!("output_2{suffix}"))).unwrap();
    let should = "@ERR12828869.1 A00627:18:HGV7TDSXX:3:1101:10004:10269/2
GCCTGGTGGATCTCTGTGAGCACCACTGAGTGATCTGTGCAGGGTATTAACCAACAGCAGACTTCCAGGATTTCCTGAGGCTGGCAAGGGTTCCTGAACCAGTTACCACTCCTTCTTGCCAGTCTAACAGGGTGGGAAAGTCCGAGCCTT
+
:FFFFFF:FFFF:F,FFFFFFFFFFF,:FFFFFFFF,FF:FF::FFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFF,FF:FFFFFFFFFF:F:FFFFFFFFFFFFFFFF:FFFFFFFF:FF,:FFFFFFFFFFFFFF,FFFFF,FFFFFF
@ERR12828869.2 A00627:18:HGV7TDSXX:3:1101:10004:13401/2
TTACTCTGTAGCATAGGCTGACTTTGAACTTAGAGTAATTTCTCCTACCTCCGTGTGCTGAGTGCCGAGGCTACAGGTGTGTGCCATCATATCCAACTTTCATGTAAGCTCTTAGCCACTAGCATTACATCGCGTAAAACCACATCAAAT
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFF:FFFFFFFFFF:FFFF:FFFFFFFFFFFFF:FFFFFFFFFF:FFFFF:FFFFFFFFFFFF
@ERR12828869.3 A00627:18:HGV7TDSXX:3:1101:10004:14998/2
CAATCATAGACTTTAATTATTAATGGACATTTCTGATTTGTTGGTTTCGGTCTATAGGTGCTGGTTGAAGAACAGAGCTCAGAGAGAAACAGTCCTAATTTGACTATTGGAAAAACAGCTGCTCCCAGGCTCCATCTTCTACCTGTCTGC
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
@ERR12828869.4 A00627:18:HGV7TDSXX:3:1101:10004:1752/2
CATCGCTGTGCGGACGCCAGAGCCGAGCCCGCGTCGCCATGCCTCGGAAAATTGAGGAGATCAAGGACTTTCTGCTGACAGCCCGGCGGAAGGATGCCAAGTCTGTCAAGATCAAGAAGAACAAGGATAATGTGAAGTTCAAGGTTCGCT
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFF:FFFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
@ERR12828869.5 A00627:18:HGV7TDSXX:3:1101:10004:17534/2
CTGGAATCCCCGCCGAAAGGTGGTGGCGTGGAACAGTAGGACTATCTCTGCCTCAAACACTGAGCAGATGGTGGGATTCATCTCGGGACTCACCATGACCATGCCCTTGCGAAGCAATGCACGGTCAAAGTCTCCAAGGGCGAGTGTAGC
+
FFFFF:FFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFF
";
    compare_fastq(&actual, should);
}

#[test]
fn test_zstd_input() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']
    read2 = ['sample_data/ERR12828869_10k_2.fq.zst']

[[step]]
    action='Head'
    n = 5

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_2.fq").exists());
    test_860_head_5(&td, ".fq");
}

#[test]
fn test_zstd_input_zst_output() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']
    read2 = ['sample_data/ERR12828869_10k_2.fq.zst']

[[step]]
    action='Head'
    n = 5

[output]
    prefix = 'output'
    format = 'Zst'
");
    test_860_head_5(&td, ".fq.zst");
}

#[test]
fn test_zstd_input_gzip_output() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']
    read2 = ['sample_data/ERR12828869_10k_2.fq.zst']

[[step]]
    action='Head'
    n = 5

[output]
    prefix = 'output'
    format = 'Gzip'
");
    test_860_head_5(&td, ".fq.gz");
}

#[test]
fn test_zstd_input_read_swap() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']
    read2 = ['sample_data/ERR12828869_10k_2.fq.zst']

[[step]]
    action='Head'
    n = 5

[[step]]
    action = 'SwapR1AndR2'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_2.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    let should = "@ERR12828869.1 A00627:18:HGV7TDSXX:3:1101:10004:10269/1
ATTGAGTACAAAAAACCTTACATAAATTAAAGAATGAATACATTTACAGGTGTCGATGCAAACGTTCCCAACTCAAGGCAACTAACAACCGATGGTGGTCAGGAGGGAAGAAACCAGAACTGAAACTGGGTCCTAAGGCTCGGACTTTCC
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFF,FF:FFFFF::FFFFFFFF:FFFFFFFF:FFFFFFFFFFFFFF,FFF:FFFFFFFFFFF,FFFFFFFFFFF,FFFFFFFFFFFFFFFFFFF:FFF,
@ERR12828869.2 A00627:18:HGV7TDSXX:3:1101:10004:13401/1
ACTATGTAAGGCTGTCGTTTTACATAGTTTTAATGAGGAAACGATTGCTTTCCACTTGTGATCTGAGCCACTGACATAGACTGTGCACAAATACTGTAGACATTCCTCTAGAGTCTGAGGTAGCATGGGTCAAAGGCCAACATGACAGTC
+
FFFFFFFFFFFFFFFFFF,FFFF:FFF:FFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFF:FFFFFF:FFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFF:FFF::FFFF:F:FF,FFFFFFFFFFFFFFFFFFFFFFFF,FF:F:FF
@ERR12828869.3 A00627:18:HGV7TDSXX:3:1101:10004:14998/1
CACCTTTCCCCTTCCTGTCACTCATGTGGACCTCATATAAGGGAAAGATACTCTCAACCTCTTGTATTTGGAGAGTTTTGAGCAGACAGGTAGAAGATGGAGCCTGGGAGCAGCTGTTTTTCCAATAGTCAAATTAGGACTGTTTCTCTC
+
FFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFF:FFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
@ERR12828869.4 A00627:18:HGV7TDSXX:3:1101:10004:1752/1
CGCAGAGGGCTGGTTCATTTCAGATCCTTCACTGCCAAACCCGGGGGTAGGGACTGCTTCAGCTTCTCTGCCTTTTCCTTGTCTGTGATAACCAGGGTGTAAAGGTACCTGCTGCAGCGAACCTTGAACTTCACATTATCCTTGTTCTTC
+
FFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFF,FF
@ERR12828869.5 A00627:18:HGV7TDSXX:3:1101:10004:17534/1
CTGGTGGTAGGCCCGACAGATGATGGCTGTTTCTTGGAGCTGAGGGTATGCAGCATCCAGCGCAACCGCTCTGCGTGTCGTGTTCTTCGAGCAGGTCAGGCTGCTACACTCGCCCTTGGAGACTTTGACCGTGCATTGCTTCGCAAGGGC
+
FFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
";
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@ERR12828869.1 A00627:18:HGV7TDSXX:3:1101:10004:10269/2
GCCTGGTGGATCTCTGTGAGCACCACTGAGTGATCTGTGCAGGGTATTAACCAACAGCAGACTTCCAGGATTTCCTGAGGCTGGCAAGGGTTCCTGAACCAGTTACCACTCCTTCTTGCCAGTCTAACAGGGTGGGAAAGTCCGAGCCTT
+
:FFFFFF:FFFF:F,FFFFFFFFFFF,:FFFFFFFF,FF:FF::FFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFF,FF:FFFFFFFFFF:F:FFFFFFFFFFFFFFFF:FFFFFFFF:FF,:FFFFFFFFFFFFFF,FFFFF,FFFFFF
@ERR12828869.2 A00627:18:HGV7TDSXX:3:1101:10004:13401/2
TTACTCTGTAGCATAGGCTGACTTTGAACTTAGAGTAATTTCTCCTACCTCCGTGTGCTGAGTGCCGAGGCTACAGGTGTGTGCCATCATATCCAACTTTCATGTAAGCTCTTAGCCACTAGCATTACATCGCGTAAAACCACATCAAAT
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFF:FFFFFFFFFF:FFFF:FFFFFFFFFFFFF:FFFFFFFFFF:FFFFF:FFFFFFFFFFFF
@ERR12828869.3 A00627:18:HGV7TDSXX:3:1101:10004:14998/2
CAATCATAGACTTTAATTATTAATGGACATTTCTGATTTGTTGGTTTCGGTCTATAGGTGCTGGTTGAAGAACAGAGCTCAGAGAGAAACAGTCCTAATTTGACTATTGGAAAAACAGCTGCTCCCAGGCTCCATCTTCTACCTGTCTGC
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
@ERR12828869.4 A00627:18:HGV7TDSXX:3:1101:10004:1752/2
CATCGCTGTGCGGACGCCAGAGCCGAGCCCGCGTCGCCATGCCTCGGAAAATTGAGGAGATCAAGGACTTTCTGCTGACAGCCCGGCGGAAGGATGCCAAGTCTGTCAAGATCAAGAAGAACAAGGATAATGTGAAGTTCAAGGTTCGCT
+
FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,FFFFFFF:FFFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
@ERR12828869.5 A00627:18:HGV7TDSXX:3:1101:10004:17534/2
CTGGAATCCCCGCCGAAAGGTGGTGGCGTGGAACAGTAGGACTATCTCTGCCTCAAACACTGAGCAGATGGTGGGATTCATCTCGGGACTCACCATGACCATGCCCTTGCGAAGCAATGCACGGTCAAAGTCTCCAAGGGCGAGTGTAGC
+
FFFFF:FFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFF:FFFFFFFFFFFFFFFFFFFFFF
";
    assert_eq!(should, actual);
}
#[test]
#[should_panic(
    expected = "Read2 is not defined in the input section, but used by transformation SwapR1AndR2"
)]
fn test_zstd_input_read_swap_no_read2() {
    //
    let _ = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']

[[step]]
    action='Head'
    n = 5

[[step]]
    action = 'SwapR1AndR2'

[output]
    prefix = 'output'
");
}

#[test]
fn test_cut_start() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'CutStart'
    n = 3
    target = 'Read1'
[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
CTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CDCCCCCCCCCC?A???###############################
@Read2
GATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CBCCCCCBCCDC?CAC=#@@A@##########################
@Read3
CACTGCTGCTTGTGGCTNTCCTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCC=@@B@#C>C?##########################
@Read4
AGTTGATCTCATCCTGANGAGCATNNNNNNNNNNNNNNNNNNNNNNNN
+
CC@CCCBCCCCCCC@?C#AAAA##########################
@Read5
AAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCCCC#ABBB##########################
@Read6
TATTACTTTGTACTTCCNATGGAGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCCCC#CCCA##########################
@Read7
GTGGGGTGGATAGTGAGNTGGAGGNNNNNNNNNNNNNNNNNNNNNNNN
+
CACC>>6CB=CABA@AB#5AA###########################
@Read8
TCAGTATGTCAGCACAANGATAATNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCC@CC@=@?@#A=@###########################
@Read9
GAGAGGTCAGTGCGATGNGAAAAANNNNNNNNNNNNNNNNNNNNNNNN
+
>CBCCCBCCCCC@@@@?#?B@B##########################
@Read10
TGAAGCTTTTTGGAAAANCTTTGANNNNNNNNNNNNNNNNNNNNNNNN
+
CCCDCCCCCCCCABBBA#BBBB##########################
";
    assert_eq!(should, actual);
}

#[test]
fn test_cut_end() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    target = 'Read1'
    action = 'CutEnd'
    n = 2
[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???#############################
@Read2
GGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@########################
@Read3
GTGCACTGCTGCTTGTGGCTNTCCTTTNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCC=@@B@#C>C?########################
@Read4
GGAAGTTGATCTCATCCTGANGAGCATNNNNNNNNNNNNNNNNNNNNNN
+
CCCCC@CCCBCCCCCCC@?C#AAAA########################
@Read5
TTCAAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNN
+
BCCCCCCCCCCCCCCCCCCC#ABBB########################
@Read6
GCTTATTACTTTGTACTTCCNATGGAGNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCCCCCCC#CCCA########################
@Read7
CGGGTGGGGTGGATAGTGAGNTGGAGGNNNNNNNNNNNNNNNNNNNNNN
+
CCCCACC>>6CB=CABA@AB#5AA#########################
@Read8
GGTTCAGTATGTCAGCACAANGATAATNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCC@CC@=@?@#A=@#########################
@Read9
CTGGAGAGGTCAGTGCGATGNGAAAAANNNNNNNNNNNNNNNNNNNNNN
+
CBB>CBCCCBCCCCC@@@@?#?B@B########################
@Read10
ATGTGAAGCTTTTTGGAAAANCTTTGANNNNNNNNNNNNNNNNNNNNNN
+
BCCCCCDCCCCCCCCABBBA#BBBB########################
";
    assert_eq!(should, actual);
}

#[test]
fn test_max_len() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'MaxLen'
    n = 5
    target='Read1'
[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
CTCCT
+
CCCCD
@Read2
GGCGA
+
CCBCB
@Read3
GTGCA
+
CCCCC
@Read4
GGAAG
+
CCCCC
@Read5
TTCAA
+
BCCCC
@Read6
GCTTA
+
CCCCC
@Read7
CGGGT
+
CCCCA
@Read8
GGTTC
+
CCCCC
@Read9
CTGGA
+
CBB>C
@Read10
ATGTG
+
BCCCC
";
    assert_eq!(should, actual);
}

#[test]
fn test_prefix_and_postfix() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'Head'
    n = 1
[[step]]
    action = 'Prefix'
    target = 'Read1'
    seq = 'ACGT'
    qual = 'ABCD'
[[step]]
    action = 'Postfix'
    target = 'Read1'
    seq = 'TGCA'
    qual = 'dcba'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
ACGTCTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNNTGCA
+
ABCDCCCCDCCCCCCCCCC?A???###############################dcba
";
    assert_eq!(should, actual);
}

#[test]
fn test_reverse() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'Head'
    n = 1
[[step]]
    action = 'ReverseComplement'
    target = 'Read1'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
NNNNNNNNNNNNNNNNNNNNNNNNCATGAGNAGAAAGTTGATGTGCAGGAG
+
###############################???A?CCCCCCCCCCDCCCC
";
    assert_eq!(should, actual);
}
#[test]
fn test_trim_poly_tail_n() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'


[[step]]
    action = 'Head'
    n = 2

[[step]]
    action = 'TrimPolyTail'
    min_length = 24
    target = 'Read1'
    base = 'N'
    max_mismatch_rate = 0
    max_consecutive_mismatches = 3


[output]
    output_hash = true
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@ERR664392.1 GAII02_0001:7:1:1116:18963#0/1
CTCCTGCACATCAACTTTCTNCTCATG
+
CCCCDCCCCCCCCCC?A???#######
@ERR664392.2 GAII02_0001:7:1:1116:17204#0/1
GGCGATTTCAATGTCCAAGGNCAGTTT
+
CCBCBCCCCCBCCDC?CAC=#@@A@##
";
    assert_eq!(should, actual);

    let actual_hash_read1 = std::fs::read_to_string(td.path().join("output_1.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash_read1,
        "6005803a2e94a795890683e0cc6140ad9b42be0f772e5cf8fff109ab713e8b4b"
    );
}

#[test]
fn test_filter_min_len() {
    //
    let td = run("
[input]
    read2 = 'sample_data/ten_reads_of_var_sizes.fq'
    read1 = 'sample_data/ten_reads.fq'
    index1 = 'sample_data/ten_reads.fq'
    index2 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'FilterMinLen'
    n = 9
    target = 'Read2'


[output]
    prefix = 'output'
    keep_index = true
    output_hash = true
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    let should = "@Read9
CTGGAGAGG
+
CBB>CBCCC
@Read10
ATGTGAAGCT
+
BCCCCCDCCC
";
    assert_eq!(should, actual);
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read9
CTGGAGAGGTCAGTGCGATGNGAAAAANNNNNNNNNNNNNNNNNNNNNNNN
+
CBB>CBCCCBCCCCC@@@@?#?B@B##########################
@Read10
ATGTGAAGCTTTTTGGAAAANCTTTGANNNNNNNNNNNNNNNNNNNNNNNN
+
BCCCCCDCCCCCCCCABBBA#BBBB##########################
";
    assert_eq!(should, actual);

    td.path()
        .read_dir()
        .unwrap()
        .for_each(|x| println!("{x:?}"));
    let actual_hash_read1 = std::fs::read_to_string(td.path().join("output_1.fq.sha256")).unwrap();
    let actual_hash_read2 = std::fs::read_to_string(td.path().join("output_2.fq.sha256")).unwrap();
    let actual_hash_index1 =
        std::fs::read_to_string(td.path().join("output_i1.fq.sha256")).unwrap();
    let actual_hash_index2 =
        std::fs::read_to_string(td.path().join("output_i2.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash_read1,
        "a058aca8c6ee9b4ebbc8c6ef212efd5e78a6eac99cebc94d74eefa71a9237b04"
    );
    assert_eq!(
        actual_hash_read2,
        "54bd4bb471ad2efeb4a39876ccf799fe58a45be9747f0e17756657957200cfb2"
    );
    assert_eq!(
        actual_hash_index1,
        "a058aca8c6ee9b4ebbc8c6ef212efd5e78a6eac99cebc94d74eefa71a9237b04"
    );
    assert_eq!(
        actual_hash_index2,
        "a058aca8c6ee9b4ebbc8c6ef212efd5e78a6eac99cebc94d74eefa71a9237b04"
    );
}

#[test]
fn test_filter_max_len() {
    //
    let td = run("
[input]
    index1 = 'sample_data/ten_reads_of_var_sizes.fq'
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads.fq'
    index2 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'FilterMaxLen'
    n = 3
    target = 'Index1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@Read2
GGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
@Read3
GTGCACTGCTGCTTGTGGCTNTCCTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCC=@@B@#C>C?##########################
";
    assert_eq!(should, actual);
}

#[test]
fn test_trim_qual_start() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'Skip'
    n = 4
[[step]]
    action = 'Head'
    n = 1

[[step]]
    action = 'TrimQualityStart'
    min = 'C'
    target = 'Read1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read5
TCAAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCCCCCCCCCCCC#ABBB##########################
";
    assert_eq!(should, actual);
}

#[test]
fn test_trim_qual_end() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true
    block_size = 3

[[step]]
    action = 'Skip'
    n = 9

[[step]]
    action = 'TrimQualityEnd'
    min = 'C'
    target = 'Read1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read10
ATGTGAAGCTTTTTG
+
BCCCCCDCCCCCCCC
";
    assert_eq!(should, actual);
}

#[test]
fn test_filter_avg_quality() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'


[[step]]
    action = 'FilterMeanQuality'
    min = 49.9
    target = 'Read1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read5\nTTCAAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNNNN\n+\nBCCCCCCCCCCCCCCCCCCC#ABBB##########################\n@Read6\nGCTTATTACTTTGTACTTCCNATGGAGNNNNNNNNNNNNNNNNNNNNNNNN\n+\nCCCCCCCCCCCCCCCCCCCC#CCCA##########################\n";
    assert_eq!(should, actual);
}

#[test]
fn test_convert_phred() {
    //
    let td = run("
[input]
    read1 = 'sample_data/test_phred.fq'

[[step]]
    action = 'ConvertPhred64To33'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
";
    assert_eq!(should, actual);
}
#[test]
fn test_convert_phred_multi() {
    //
    let td = run("
[input]
    read1 = 'sample_data/test_phred.fq'
    read2 = 'sample_data/test_phred.fq'
    index1 = 'sample_data/test_phred.fq'
    index2 = 'sample_data/test_phred.fq'

[[step]]
    action = 'ConvertPhred64To33'


[output]
    prefix = 'output'
    keep_index = true

[options]
    accept_duplicate_files = true
");
    let should = "@Read1
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
";
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_i1.fq")).unwrap();
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_i2.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_filter_qualified_bases() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'


[[step]]
    action = 'FilterQualifiedBases'
    min_quality='C'
    min_percentage = 0.37
    target = 'Read1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read5\nTTCAAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNNNN\n+\nBCCCCCCCCCCCCCCCCCCC#ABBB##########################\n@Read6\nGCTTATTACTTTGTACTTCCNATGGAGNNNNNNNNNNNNNNNNNNNNNNNN\n+\nCCCCCCCCCCCCCCCCCCCC#CCCA##########################\n";
    assert_eq!(should, actual);
}

#[test]
fn test_filter_too_many_n() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads_var_n.fq'


[[step]]
    action = 'FilterTooManyN'
    n = 25
    target = 'read1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read4N\nGGAAGTTGATCTCATCCTGANGAGCATNNNNNNNNNNNNNNNNNNNNNNNN\n+\nCCCCC@CCCBCCCCCCC@?C#AAAA##########################\n@Read5N\nTTCAAATCCATCTTTGGATANTTCCCTNNNNNNNNNNNNNNNNNNNNNNNN\n+\nBCCCCCCCCCCCCCCCCCCC#ABBB##########################\n";
    assert_eq!(should, actual);
}

#[test]
fn test_subsample() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'


[[step]]
    action = 'FilterSample'
    p = 0.25
    seed  = 42


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read4\nGGAAGTTGATCTCATCCTGANGAGCATNNNNNNNNNNNNNNNNNNNNNNNN\n+\nCCCCC@CCCBCCCCCCC@?C#AAAA##########################\n@Read7\nCGGGTGGGGTGGATAGTGAGNTGGAGGNNNNNNNNNNNNNNNNNNNNNNNN\n+\nCCCCACC>>6CB=CABA@AB#5AA###########################\n";
    assert_eq!(should, actual);
}

#[test]
fn test_order_maintained_in_single_core_transforms() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']

 [options]
    block_size = 100
    thread_count = 8


[[step]]
    action = '_InternalDelay'

[[step]]
    action='Skip'
    n = 500

[[step]]
    action='Head'
    n = 500

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    assert!(!td.path().join("output_2.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.fq").unwrap();
    assert!(should == actual);

    //panic!("Should not be reached");
}

#[test]
fn test_dedup() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'


[[step]]
    action = 'FilterDuplicates'
    false_positive_rate = 0.001
    target = 'Read1'
    seed = 34

[output]
    prefix = 'output'

");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //check line count
    assert_eq!(actual.lines().count() / 4, 10000 - 787);
}

#[test]
fn test_dedup_exact() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'


[[step]]
    action = 'FilterDuplicates'
    false_positive_rate = 0.0
    target = 'Read1'
    seed = 34

[output]
    prefix = 'output'

");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 10000 - 787);
    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 10000 - 787);
}
#[test]
fn test_dedup_read2() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'


[[step]]
    action = 'FilterDuplicates'
    false_positive_rate = 0.001
    target = 'Read2'
    seed = 34

[output]
    prefix = 'output'

");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //check line count
    assert_eq!(actual.lines().count() / 4, 10000 - 769);
}

#[test]
fn test_dedup_read_combo() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'FilterDuplicates'
    false_positive_rate = 0.001
    target = 'all'
    seed = 34


[output]
    prefix = 'output'

");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //check line count
    assert_eq!(actual.lines().count() / 4, 10000 - 596);
}

#[test]
fn test_dedup_read_combo_incl_indndex() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
    index1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    index2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'FilterDuplicates'
    false_positive_rate = 0.001
    target = 'all'
    seed = 34

[options]
    accept_duplicate_files = true


[output]
    prefix = 'output'

");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //check line count
    assert_eq!(actual.lines().count() / 4, 10000 - 596); // same as read1/read2 I suppose
}

#[test]
fn test_low_complexity_filter() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.head_500.fq'

[[step]]
    action = 'FilterLowComplexity'
    target = 'Read1'
    threshold = 0.6


[output]
    prefix = 'output'

");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = std::fs::read_to_string(
        "sample_data/ERR12828869_10k_1.head_500.fq.fastp.complexity_filter.fq",
    )
    .unwrap();

    assert_eq!(should, actual);
}

#[test]
fn test_quantify_regions_simple() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[[step]]
    action = 'QuantifyRegions'
    infix = 'kmer'
    regions = [
            { source = 'Read1', start = 6, length = 6}
    ]
    separator = '_'

[output]
    prefix = 'output'

");
    assert!(td.path().join("output_kmer.qr.json").exists());
    let actual = std::fs::read_to_string(td.path().join("output_kmer.qr.json")).unwrap();
    let should = std::fs::read_to_string("sample_data/ERR664392_1250.fq.quantify.json").unwrap();

    let json_actual = serde_json::from_str::<serde_json::Value>(&actual).unwrap();
    let json_should = serde_json::from_str::<serde_json::Value>(&should).unwrap();

    assert_eq!(json_should, json_actual);
}

#[test]
fn test_quantify_regions_multi() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'QuantifyRegions'
    infix = 'kmer'
    regions = [
            { source = 'Read1', start = 6, length = 6},
            { source = 'Read2', start = 10, length = 7}
    ]
    separator = 'xyz'

[output]
    prefix = 'output'

");
    assert!(td.path().join("output_kmer.qr.json").exists());
    let actual = std::fs::read_to_string(td.path().join("output_kmer.qr.json")).unwrap();
    let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.quantify.json").unwrap();

    let json_actual: std::collections::HashMap<String, usize> =
        serde_json::from_str::<_>(&actual).unwrap();
    let json_should: std::collections::HashMap<String, usize> =
        serde_json::from_str::<_>(&should).unwrap();
    assert_eq!(json_actual, json_should);
}

#[test]
fn test_trim_poly_tail_detail() {
    //
    let td = run("
[input]
    read1 = 'sample_data/test_trim.fq'

[[step]]
    action = 'TrimPolyTail'
    min_length = 10
    target = 'Read1'
    base = '.'
    max_mismatch_rate = 0.09
    max_consecutive_mismatches = 3

[[step]]
    action = 'FilterMinLen'
    target = 'Read1'
    n = 14



[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(!actual.contains("Read1\n"));
    assert!(!actual.contains("Read3\n"));
    assert!(!actual.contains("Read5\n"));
    assert!(!actual.contains("Read7\n"));
    assert!(!actual.contains("Read9\n"));

    assert!(actual.contains("Read2\n"));
    assert!(actual.contains("Read4\n"));
    assert!(actual.contains("Read6\n"));
    assert!(actual.contains("Read8\n"));
    assert!(actual.contains("Read10\n"));
}

#[test]
fn test_trim_poly_tail_detail_g() {
    //
    let td = run("
[input]
    read1 = 'sample_data/test_trim.fq'

[[step]]
    action = 'TrimPolyTail'
    min_length = 10
    target = 'Read1'
    base = 'G'
    max_mismatch_rate = 0.11
    max_consecutive_mismatches = 3

[[step]]
    action = 'FilterMinLen'
    target = 'Read1'
    n = 14



[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(!actual.contains("Read5\n"));
    assert!(!actual.contains("Read6\n"));

    assert!(actual.contains("Read1\n"));
    assert!(actual.contains("Read2\n"));
    assert!(actual.contains("Read3\n"));
    assert!(actual.contains("Read4\n"));
    assert!(actual.contains("Read7\n"));
    assert!(actual.contains("Read8\n"));
    assert!(actual.contains("Read9\n"));
    assert!(actual.contains("Read10\n"));
}

#[test]
fn test_filter_empty() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads_of_var_sizes.fq'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'CutStart'
    n = 5
    target = 'Read1'

[[step]]
    action = 'FilterEmpty'
    target = 'Read1'


[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(actual.contains("Read6\n"));
    assert!(actual.contains("Read7\n"));
    assert!(actual.contains("Read8\n"));
    assert!(actual.contains("Read9\n"));
    assert!(actual.contains("Read10\n"));

    assert!(!actual.contains("Read1\n"));
    assert!(!actual.contains("Read2\n"));
    assert!(!actual.contains("Read3\n"));
    assert!(!actual.contains("Read4\n"));
    assert!(!actual.contains("Read5\n"));
}

#[test]
fn test_trim_poly_tail_long() {
    //
    let td = run("
[input]
    read1 = 'sample_data/test_trim_long.fq'

[[step]]
    action = 'TrimPolyTail'
    min_length = 10
    target = 'Read1'
    base = 'A'
    max_mismatch_rate = 0.10
    max_consecutive_mismatches = 3

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1
AGTC
+
CCCC
@Read2
AGTC
+
CCCC
";
    assert_eq!(should, actual);
}

fn compare_fastq(actual: &str, should: &str) {
    if actual != should {
        //write both to a temp file, run diff on themm
        let mut tf_actual = tempfile::NamedTempFile::new().unwrap();
        let mut tf_should = tempfile::NamedTempFile::new().unwrap();
        tf_actual.write_all(actual.as_bytes()).unwrap();
        tf_should.write_all(should.as_bytes()).unwrap();
        let output = std::process::Command::new("diff")
            .arg(tf_should.path())
            .arg(tf_actual.path())
            .output()
            .unwrap();
        println!(
            "{}",
            std::str::from_utf8(&output.stdout)
                .unwrap()
                .replace("< ", "should ")
                .replace('>', "actual")
        );
    }
    assert_eq!(should, actual);
}

#[test]
fn test_trim_adapter_mismatch_tail() {
    //
    let td = run("
[input]
    read1 = 'sample_data/test_trim_adapter_tail.fq'

[[step]]
    action = 'TrimAdapterMismatchTail'
    query = 'GATCGGAAGAGCACACGTCTGAACTCCAGTCAC'
    min_length = 12
    max_mismatches = 0
    target = 'Read1'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //copy the expected output
    let mut fh = std::fs::File::create("debug.fq").unwrap();
    fh.write_all(actual.as_bytes()).unwrap();
    let should = "@Read1
GTGTGTTATAAGTGCGGTTGTGTGTGTATGTGTGTGTGTGTGTGTCAGACTACCCTAATTGTAACCATATCTCTGGTTCCCATTAAAAAACATCATTTTAGTTAAAAAAAAAAAAAAAAAA
+
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
@Read2
GTGTTGTATAGCTTCGGGGGCTGGGATGCCGTGTACACACGCACAAGTACACATCGCGCTCAGGACTTCACTGAAGATTCACGTGCAATTGAACGCTTCATTAAACAAAAGAAAACCTCAAAAAAAAAAAAAAAAAAA
+
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
@Read3
ATGGTGTATAGTGTGGTATATTTATACAATGTGGAATGAATAAATCAAAGTATATACTTCAGTAAGAGCACAAAAAAAAAAAAAAAAAAGATCGGAAGAGCACACGTCTGAACTCCAGTCACTCAGGAATCTCGTATGCCGTCTTCTGCT
+
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
@Read4
TATGGTTATAAGGTATGCTGGGTTCTCACTGAGGTTATTTAAATAAAGCTTAAGGTTATTTGCTTGGTGTGTTTTTCATAAACATTTTCCTGCCTTAGAAAAAAAAAAAAAAAAAAA
+
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
@Read5
CTTGTGTATATGATTGTATGGACTTGTTGATGCATGTAAACTGGGTGCATTCTGTTGCCTCTGTATGTTAAATAGTGACCAATGTTTTTACGAAAGAATTGAACAAAAAAATATCTTTTAAGAAAAAAAAAAAAAAAAAAGATCGGAAGA
+
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
@Read6
GTCTTCTATATTGCTGTGTTTTGGGCAGACCAATCTTCTATCAGTCACAGAAAACAACCTGTTAATTCTTTTTTCTTCTTTTTTTAAGTATCTATTAAACGTGAATTCTGAGAAAAAAAAAAAAAAAAAAAA
+
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
"
;
    compare_fastq(&actual, should);
}

#[test]
fn test_gzip_blocks_spliting_reads() {
    //
    use std::io::Read;
    let td = run("
[input]
    read1 = 'sample_data/test_gzip_block_unaligned.fastq.gz'

[options]
    buffer_size = 100

[[step]]
    action = 'Report'
    label = 'report'

[output]
    prefix = 'output'
    report_json = true
");
    assert!(td.path().join("output.json").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let mut raw = Vec::new();
    std::fs::File::open("sample_data/test_gzip_block_unaligned.fastq.gz")
        .unwrap()
        .read_to_end(&mut raw)
        .unwrap();
    let (mut reader, _compression) = niffler::get_reader(Box::new(&raw[..])).unwrap();
    let mut should = String::new();
    reader.read_to_string(&mut should).unwrap();
    assert_eq!(should, actual);
}


#[test]
fn test_interleaved_output() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'Head'
    n = 10


[output]
    prefix = 'output'
    interleave = true

");

    assert!(!td.path().join("output_1.fq").exists());
    assert!(!td.path().join("output_2.fq").exists());

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_interleaved.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 20);

    let lines: Vec<_> = actual.split('\n').collect();
    let mut last = None;
    for ii in (0..21).step_by(4) {
        let read = lines[ii];
        if let Some(slast) = last {
            assert_eq!(slast, read.replace("/2", "/1"));
            last = None;
        } else {
            last = Some(read.to_string());
        }
    }
}

#[test]
fn test_stdout_output() {
    //
    let (td, stdout, stderr) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[step]]
    action = 'Head'
    n = 10

[output]
    prefix = 'output'
    stdout = true

",
    );
    dbg!(&stdout);
    dbg!(&stderr);

    assert!(!td.path().join("output_1.fq").exists());
    assert!(!td.path().join("output_2.fq").exists());
    assert!(!td.path().join("output_interleaved.fq").exists());

    //check head
    let actual = stdout;
    assert_eq!(actual.lines().count() / 4, 10);
}

#[test]
fn test_stdout_output_interleaved() {
    //
    let (td, stdout, stderr) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'Head'
    n = 10


[output]
    prefix = 'output'
    stdout = true

",
    );
    dbg!(&stdout);
    dbg!(&stderr);

    assert!(!td.path().join("output_1.fq").exists());
    assert!(!td.path().join("output_2.fq").exists());
    assert!(!td.path().join("output_interleaved.fq").exists());

    //check head
    let actual = stdout;
    assert_eq!(actual.lines().count() / 4, 20);

    //test automatic interleaving

    let lines: Vec<_> = actual.split('\n').collect();
    let mut last = None;
    for ii in (0..21).step_by(4) {
        let read = lines[ii];
        if let Some(slast) = last {
            assert_eq!(slast, read.replace("/2", "/1"));
            last = None;
        } else {
            last = Some(read.to_string());
        }
    }
}

#[test]
fn test_input_interleaved() {
    //
    let (td, _stdout, _stderr) = run_and_capture(
        "
[input]
    read1 = 'sample_data/interleaved.fq.zst'
    interleaved = true

[[step]]
    action = 'Head'
    n = 5


[output]
    prefix = 'output'

",
    );

    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_2.fq").exists());
    assert!(!td.path().join("output_interleaved.fq").exists());
    test_860_head_5(&td, ".fq");
}

#[test]
fn test_input_interleaved_test_premature_termination() {
    //
    let (td, _stdout, _stderr) = run_and_capture(
        "
[input]
    read1 = 'sample_data/interleaved.fq.zst'
    interleaved = true


[options]
    block_size = 2


[[step]]
    action = 'Head'
    n = 4


[output]
    prefix = 'output'

",
    );

    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_2.fq").exists());
    assert!(!td.path().join("output_interleaved.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count(), 16);
    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    assert_eq!(actual.lines().count(), 16);
}

#[test]
#[should_panic(expected = "Block size must be even for interleaved input.")]
fn test_interleaved_must_have_even_block_size() {
    //
    let _ = run("
[input]
    read1 = 'sample_data/interleaved.fq.zst'
    interleaved = true


[options]
    block_size = 1


[[step]]
    action = 'Head'
    n = 2


[output]
    prefix = 'output'

");
}

#[test]
fn test_filter_other_file_keep() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[step]]
    action = 'FilterOtherFile'
    filename = 'sample_data/ERR12828869_10k_1.head_500.fq'
    keep_or_remove = 'Keep'
    false_positive_rate = 0
    seed  = 42

[output]
    prefix = 'output'
",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count(), 500 * 4);
    let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.fq").unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_filter_other_file_remove() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[step]]
    action = 'FilterOtherFile'
    filename = 'sample_data/ERR12828869_10k_1.head_500.fq'
    keep_or_remove = 'Remove'
    false_positive_rate = 0.000001 # so we trigger the other code path
    seed  = 42

[output]
    prefix = 'output'
",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.fq").unwrap();
    assert_eq!(actual.lines().count(), 9500 * 4);
}

#[test]
fn test_filter_other_file_remove_bam() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[step]]
    action = 'FilterOtherFile'
    filename = 'sample_data/ERR12828869_10k_1.head_500.bam'
    keep_or_remove = 'Remove'
    ignore_unaligned = true
    false_positive_rate = 0
    seed  = 42
    readname_end_chars = ' '


[output]
    prefix = 'output'
",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.fq").unwrap();
    assert_eq!(actual.lines().count(), (10000 - 490) * 4);
}

#[test]
fn test_filter_other_file_remove_bam_unaligned() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[step]]
    action = 'FilterOtherFile'
    filename = 'sample_data/ERR12828869_10k_1.head_500.all_unaligned.bam'
    keep_or_remove = 'Remove'
    ignore_unaligned = true
    false_positive_rate = 0
    seed  = 42
    readname_end_chars = ' '


[output]
    prefix = 'output'
",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.fq").unwrap();
    assert_eq!(actual.lines().count(), (10000 - 0) * 4);
}

#[test]
fn test_filter_other_file_remove_bam_unaligned_no_ignore() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[step]]
    action = 'FilterOtherFile'
    filename = 'sample_data/ERR12828869_10k_1.head_500.all_unaligned.bam'
    keep_or_remove = 'Remove'
    ignore_unaligned = false
    false_positive_rate = 0
    seed  = 42
    readname_end_chars = ' '


[output]
    prefix = 'output'
",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    //let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.fq").unwrap();
    assert_eq!(actual.lines().count(), (10000 - 490) * 4);
}


#[test]
fn test_rename_regex() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/mgi/oldschool.fq'
    read2 = 'sample_data/mgi/oldschool.fq'
    index1 = 'sample_data/mgi/oldschool.fq'
    index2 = 'sample_data/mgi/oldschool.fq'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'Rename'
    search = '(.)/([1/2])$'
    replacement = '$1 $2'

[output]
    prefix = 'output'
    keep_index = true
",
    );
    let should = std::fs::read_to_string("sample_data/mgi/oldschool_rename.fq").unwrap();

    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual, should);

    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    assert_eq!(actual, should);

    let actual = std::fs::read_to_string(td.path().join("output_i1.fq")).unwrap();
    assert_eq!(actual, should);

    let actual = std::fs::read_to_string(td.path().join("output_i2.fq")).unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_head_with_index() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads.fq'
    index1 = 'sample_data/ten_reads.fq'
    index2 = 'sample_data/ten_reads.fq'

[[step]]
    action='Skip'
    n = 5

[options]
    block_size = 2
    accept_duplicate_files = true

[output]
    prefix = 'output'
    keep_index = true

",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 20);

    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 20);

    let actual = std::fs::read_to_string(td.path().join("output_i1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 20);

    let actual = std::fs::read_to_string(td.path().join("output_i2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 20);
}

#[test]
fn test_head_with_index_and_demultiplex() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads.fq'
    index1 = 'sample_data/ten_reads.fq'
    index2 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'Demultiplex'
    regions = [
        {source = 'index1', start=0, length=1},
    ]
    max_hamming_distance = 0
    output_unmatched = true

[step.barcode_to_name]
    C = 'C'
    A = 'A'
    G = 'G'


[[step]]
    action='Skip'
    n = 5

[options]
    block_size = 2
    accept_duplicate_files = true

[output]
    prefix = 'output'
    keep_index = true

",
    );

    let actual = std::fs::read_to_string(td.path().join("output_A_1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 1 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_G_1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_G_2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_G_i1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_G_i2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_C_1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_C_1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_C_2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_C_i1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_G_i2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 2 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_A_2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 1 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_A_i1.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 1 * 4);

    let actual = std::fs::read_to_string(td.path().join("output_A_i2.fq")).unwrap();
    assert_eq!(actual.matches('\n').count(), 1 * 4);
}

#[test]
fn test_rename_regex_shorter() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.head_500.fq'

[[step]]
    action = 'Rename'
    search = '(.)..$'
    replacement = '$1'

[output]
    prefix = 'output'
    # keep_index = true # make sure keep index doesn't make it fail in
",
    );
    let should =
        std::fs::read_to_string("sample_data/ERR12828869_10k_1.head_500.truncated_name.fq")
            .unwrap();

    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_rename_regex_gets_longer() {
    let (td, _, _) = run_and_capture(
        "
[input]
    read1 = 'sample_data/mgi/oldschool.fq'

[[step]]
    action = 'Rename'
    search = '(.+)'
    replacement = 'some_random_text $1'

[output]
    prefix = 'output'
",
    );
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = std::fs::read_to_string("sample_data/mgi/oldschool_rename_longer.fq").unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_inspect_read1() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'Skip'
    n = 10

[[step]]
    action ='Inspect'
    infix = 'tcepsni'
    n = 2
    target = 'read1'

[output]
    prefix = 'output'
    format = 'None'

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_tcepsni_1.fq")).unwrap();
    assert_eq!(2, actual.lines().count() / 4);
    //we test actual content in read2
}

#[test]
fn test_inspect_read2() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'Skip'
    n = 10

[[step]]
    action ='Inspect'
    infix = 'tcepsni'
    n = 2
    target = 'read2'

[output]
    prefix = 'output'
    format = 'None'

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_tcepsni_2.fq")).unwrap();
    assert_eq!(2, actual.lines().count() / 4);
    let should = std::fs::read_to_string("sample_data/ERR12828869_test_inspect.fq").unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_inspect_index1() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
    index1 = 'sample_data/ERR12828869_10k_2.fq.zst'
    index2 = 'sample_data/ERR12828869_10k_1.fq.zst'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'Skip'
    n = 10

[[step]]
    action ='Inspect'
    infix = 'tcepsni'
    n = 2
    target = 'index1'

[output]
    prefix = 'output'
    format = 'None'

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_tcepsni_i1.fq")).unwrap();
    assert_eq!(2, actual.lines().count() / 4);
    //not the swap
    let should = std::fs::read_to_string("sample_data/ERR12828869_test_inspect.fq").unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_inspect_index2() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
    index1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    index2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[options]
    accept_duplicate_files = true

[[step]]
    action = 'Skip'
    n = 10

[[step]]
    action ='Inspect'
    infix = 'tcepsni'
    n = 2
    target = 'index2'

[output]
    prefix = 'output'
    format = 'None'

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_tcepsni_i2.fq")).unwrap();
    assert_eq!(2, actual.lines().count() / 4);
    //not the swap
    let should = std::fs::read_to_string("sample_data/ERR12828869_test_inspect.fq").unwrap();
    assert_eq!(actual, should);
}

#[test]
fn test_very_long_reads() {
    let fastq = format!(
        "@{}\n{}\n+\n{}\n",
        "A".repeat(1000),
        "AGCT".repeat(1000000 / 4),
        "B".repeat(1000000)
    );
    let td2 = tempfile::tempdir().unwrap();
    let fq = td2.path().join("test.fq");
    std::fs::write(&fq, &fastq).unwrap();

    let td = run(&format!(
        "
        [input]
            read1 = '{}'
        [output]
            prefix = 'output'
        ",
        fq.to_string_lossy()
    ));
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(actual == fastq);
}

#[test]
fn test_mega_long_reads() {
    let fastq = format!(
        "@{}\n{}\n+\n{}\n",
        "A".repeat(10000),
        "AGCT".repeat(10_000_000 / 4),
        "B".repeat(10_000_000)
    );
    let td2 = tempfile::tempdir().unwrap();
    let fq = td2.path().join("test.fq");
    std::fs::write(&fq, &fastq).unwrap();

    let td = run(&format!(
        "
        [input]
            read1 = '{}'
        [output]
            prefix = 'output'
            output_hash = true
        ",
        fq.to_string_lossy()
    ));
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(actual == fastq);
    let actual_hash = std::fs::read_to_string(td.path().join("output_1.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "45812d8d501765790f31cf40393c08808b96dbb4054f905452fc2f98ecd0769c"
    );
}

#[test]
fn test_interleaved_output_demultiplex() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'

    index1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    index2 = 'sample_data/ERR12828869_10k_2.fq.zst'

[[step]]
    action = 'MaxLen'
    target = 'index1'
    n = 3

[[step]]
    action = 'MaxLen'
    target = 'index2'
    n = 3

[[step]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=1},
    ]
    max_hamming_distance = 0
    output_unmatched = true

[step.barcode_to_name]
    C = 'rc'
    a = 'ra'
    g = 'rg'

[options]
    output_buffer_size = 10000
    accept_duplicate_files = true

[[step]]
    action = 'Progress'
    output_infix = 'pp'

[output]
    prefix = 'output'
    interleave = true
    output_hash = true
    keep_index = true

");

    assert!(!td.path().join("output_1.fq").exists());
    assert!(!td.path().join("output_2.fq").exists());

    td.path()
        .read_dir()
        .unwrap()
        .for_each(|x| println!("{x:?}"));

    let actual = std::fs::read_to_string(td.path().join("output_ra_interleaved.fq")).unwrap();
    let count_a = actual.lines().count() / 4;
    let actual = std::fs::read_to_string(td.path().join("output_rc_interleaved.fq")).unwrap();
    let count_c = actual.lines().count() / 4;

    let actual = std::fs::read_to_string(td.path().join("output_rg_interleaved.fq")).unwrap();
    let count_g = actual.lines().count() / 4;

    let actual =
        std::fs::read_to_string(td.path().join("output_no-barcode_interleaved.fq")).unwrap();
    let count_t = actual.lines().count() / 4;
    assert_eq!(count_a + count_c + count_g + count_t, 20000);

    let actual_hash =
        std::fs::read_to_string(td.path().join("output_ra_interleaved.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "292d51fbe32a9429b106e9a0ae4f0768185e7f847140feb37355a221fd063f8e"
    );
    let actual_hash =
        std::fs::read_to_string(td.path().join("output_rc_interleaved.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "e1fef59c7019f9a4f37a1493156dc0ee48053b66744a6ee87772fd8fa3096ecd"
    );
    let actual_hash =
        std::fs::read_to_string(td.path().join("output_rg_interleaved.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "151ceff70625f98ad01949d7016cf4d344c80deefe69c63a60aa5846ae7f1820"
    );
    let actual_hash =
        std::fs::read_to_string(td.path().join("output_no-barcode_interleaved.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "20b1612a99b774ac649ac888dc27c2490269b17faff12c1dc95058935c6f15cf"
    );

    let actual_hash = std::fs::read_to_string(td.path().join("output_ra_i1.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "497fd7490a79c8f743e7614aa21e8db8248b27cdddb55dbecd2afd7c5699ac63"
    );
    let actual_hash = std::fs::read_to_string(td.path().join("output_rc_i1.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "548fd0a5397c9583d8af6427250ce9bf64e5be732fa05b3036087b611c682a73"
    );
    let actual_hash = std::fs::read_to_string(td.path().join("output_rg_i1.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "43fbcea6c0e30c9bc3c3e25a9f3dfd2b59588a6d067fd40ef0a7db55e6dc471b"
    );
    let actual_hash =
        std::fs::read_to_string(td.path().join("output_no-barcode_i1.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "47d5efdbe937480b31196d12fa927baccae2165128c252bffc807d56ae4540e1"
    );

    let actual_hash = std::fs::read_to_string(td.path().join("output_ra_i2.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "b59411c78043740938263818fd284a969da3e9214a54b15d67323541acb2de42"
    );
    let actual_hash = std::fs::read_to_string(td.path().join("output_rc_i2.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "d7fa5e723c6e7d23a9d46ca7303770237b51249888f1fdee82be0b82dcfc4b30"
    );
    let actual_hash = std::fs::read_to_string(td.path().join("output_rg_i2.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "6457320af13aef3c252f19a37147b86b6eb0ad92538a135c863d8f47151dc584"
    );
    let actual_hash =
        std::fs::read_to_string(td.path().join("output_no-barcode_i2.fq.sha256")).unwrap();
    assert_eq!(
        actual_hash,
        "667242e64bab4e5a5ca2a3339c10506e5f21a9641bcca514415f4d2066b961e6"
    );

    let actual = std::fs::read_to_string(td.path().join("output_pp.progress")).unwrap();
    dbg!(&actual);
    assert!(actual.contains("molecules for an effective rate of"));
}

#[test]
fn test_usage() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        //.join("debug")
        .join("mbf_fastq_processor");
    let cmd = std::process::Command::new(bin_path).output().unwrap();
    //let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();
    assert!(stderr.contains("Usage:"));
    assert!(!cmd.status.success());
}

/*
* difficult to test, since it only works in --release build binaries...
We're going to test it in the nix build, I suppose
#[test]
fn test_friendly_panic() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        //.join("debug")
        .join("mbf_fastq_processor");
    let cmd = std::process::Command::new(bin_path).arg("--test-friendly-panic").output().unwrap();
    //let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();
    assert!(stderr.contains("Usage:"));
    assert!(!cmd.status.success());
} */
