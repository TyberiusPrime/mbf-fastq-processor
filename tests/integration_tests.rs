use ex::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn run(config: &str) -> tempfile::TempDir {
    let td = tempdir().unwrap();
    let config_file = td.path().join("config.toml");
    let mut f = File::create(&config_file).unwrap();
    f.write_all(config.as_bytes()).unwrap();

    mbf_fastq_processor::run(&config_file, &td.path()).unwrap();

    td
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
    let should = format!("{}{}", should, should);
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

[[transform]]
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

[[transform]]
    action='Head'
    n = 5

[output] 
    prefix = 'temp'
");
    dbg!(td.path());
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
    assert_eq!(actual.chars().filter(|x| *x == '\n').count() , 5*4);
    assert_eq!(should, actual);
}

#[test]
fn test_zstd_input() {
    //
    let td = run("
[input]
    read1 = ['sample_data/ERR12828869_10k_1.fq.zst']
    read2 = ['sample_data/ERR12828869_10k_2.fq.zst']

[[transform]]
    action='Head'
    n = 5

[output] 
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_2.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
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

    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
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
fn test_cut_start() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[transform]]
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
[[transform]]
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
[[transform]]
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
[[transform]]
    action = 'Head'
    n = 1
[[transform]]
    action = 'PreFix'
    target = 'Read1'
    seq = 'ACGT'
    qual = 'ABCD'
[[transform]]
    action = 'PostFix'
    target = 'Read1'
    seq = 'TGCA'
    qual = 'dcba'

[output] 
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should ="@Read1
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
[[transform]]
    action = 'Head'
    n = 1
[[transform]]
    action = 'Reverse'
    target = 'Read1'

[output] 
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should ="@Read1
NNNNNNNNNNNNNNNNNNNNNNNNGTACTCNTCTTTCAACTACACGTCCTC
+
###############################???A?CCCCCCCCCCDCCCC
";
    assert_eq!(should, actual);
}
