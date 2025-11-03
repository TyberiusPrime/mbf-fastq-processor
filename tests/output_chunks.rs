use mbf_fastq_processor::run;
use noodles::bam::{Record as BamRecord, io::reader::Builder as BamReaderBuilder};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const READ_COUNT: usize = 5;

fn write_fastq_file(path: &Path, file_prefix: &str, name_prefix: &str) -> PathBuf {
    let mut content = String::new();
    for i in 0..READ_COUNT {
        let name = format!("@{}_{i}", name_prefix);
        content.push_str(&format!("{name}\n"));
        content.push_str(&format!("ACGT{}A\n", i % 10));
        content.push_str("+\n");
        content.push_str("IIIIII\n");
    }
    let file_path = path.join(format!("{file_prefix}.fq"));
    fs::write(&file_path, content).expect("write fastq");
    file_path
}

fn sanitize_path(path: &Path) -> String {
    path.to_str().expect("path utf8").replace('\\', "\\\\")
}

fn read_fastx_record_count(path: &Path, header_prefix: char) -> usize {
    let mut file = fs::File::open(path).expect("open output");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("read output");
    data.lines()
        .filter(|line| line.starts_with(header_prefix))
        .count()
}

fn count_bam_records(path: &Path) -> usize {
    let mut reader = BamReaderBuilder::default()
        .build_from_path(path)
        .expect("open bam");
    reader.read_header().expect("bam header");
    let mut record = BamRecord::default();
    let mut count = 0;
    while reader.read_record(&mut record).expect("bam record") != 0 {
        count += 1;
    }
    count
}

fn common_setup() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf, PathBuf) {
    let temp = tempdir().expect("tempdir");
    let input_dir = temp.path();
    let read1 = write_fastq_file(input_dir, "read1_input", "pair");
    let read2 = write_fastq_file(input_dir, "read2_input", "pair");
    let config_path = input_dir.join("config.toml");
    let output_dir = input_dir.join("output");
    fs::create_dir_all(&output_dir).expect("create output dir");
    (temp, read1, read2, config_path, output_dir)
}

#[cfg(unix)]
fn create_fifo(path: &Path) -> io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains null"))?;
    let mode = libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IWGRP;
    let result = unsafe { libc::mkfifo(c_path.as_ptr(), mode) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[test]
fn fastq_output_chunks() {
    let (_temp, read1, read2, config_path, output_dir) = common_setup();
    let config = format!(
        r#"[input]
read1 = ["{}"]
read2 = ["{}"]

[output]
prefix = "chunked_fastq"
Chunksize = 3
"#,
        sanitize_path(&read1),
        sanitize_path(&read2)
    );
    fs::write(&config_path, config).expect("write config");
    run(&config_path, &output_dir, false).expect("run processor");

    let first_chunk = output_dir.join("chunked_fastq_read1.0000.fq");
    let second_chunk = output_dir.join("chunked_fastq_read1.0001.fq");
    assert!(first_chunk.exists());
    assert!(second_chunk.exists());
    assert_eq!(read_fastx_record_count(&first_chunk, '@'), 3);
    assert_eq!(read_fastx_record_count(&second_chunk, '@'), READ_COUNT - 3);

    let read2_first = output_dir.join("chunked_fastq_read2.0000.fq");
    assert!(read2_first.exists());
    assert_eq!(read_fastx_record_count(&read2_first, '@'), 3);
}

#[test]
fn fasta_output_chunks() {
    let (_temp, read1, read2, config_path, output_dir) = common_setup();
    let config = format!(
        r#"[input]
read1 = ["{}"]
read2 = ["{}"]

[output]
prefix = "chunked_fasta"
format = "Fasta"
Chunksize = 2
"#,
        sanitize_path(&read1),
        sanitize_path(&read2)
    );
    fs::write(&config_path, config).expect("write config");
    run(&config_path, &output_dir, false).expect("run processor");

    let first_chunk = output_dir.join("chunked_fasta_read1.0000.fasta");
    let second_chunk = output_dir.join("chunked_fasta_read1.0001.fasta");
    let third_chunk = output_dir.join("chunked_fasta_read1.0002.fasta");
    assert!(first_chunk.exists());
    assert!(second_chunk.exists());
    assert!(third_chunk.exists());
    assert_eq!(read_fastx_record_count(&first_chunk, '>'), 2);
    assert_eq!(read_fastx_record_count(&second_chunk, '>'), 2);
    assert_eq!(read_fastx_record_count(&third_chunk, '>'), READ_COUNT - 4);
}

#[test]
fn bam_output_chunks() {
    let (_temp, read1, read2, config_path, output_dir) = common_setup();
    let config = format!(
        r#"[input]
read1 = ["{}"]
read2 = ["{}"]

[output]
prefix = "chunked_bam"
format = "Bam"
Chunksize = 4
"#,
        sanitize_path(&read1),
        sanitize_path(&read2)
    );
    fs::write(&config_path, config).expect("write config");
    run(&config_path, &output_dir, false).expect("run processor");

    let first_chunk = output_dir.join("chunked_bam_read1.0000.bam");
    let second_chunk = output_dir.join("chunked_bam_read1.0001.bam");
    assert!(first_chunk.exists());
    assert!(second_chunk.exists());
    assert_eq!(count_bam_records(&first_chunk), 4);
    assert_eq!(count_bam_records(&second_chunk), READ_COUNT - 4);

    let read2_first = output_dir.join("chunked_bam_read2.0000.bam");
    assert!(read2_first.exists());
    assert_eq!(count_bam_records(&read2_first), 4);
}

#[cfg(unix)]
#[test]
fn fastq_output_to_named_pipe_without_chunking() {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let (_temp, read1, read2, config_path, output_dir) = common_setup();
    let fifo_path = output_dir.join("fifo_fastq_read1.fq");
    create_fifo(&fifo_path).expect("create fifo");

    let config = format!(
        r#"[input]
read1 = ["{}"]
read2 = ["{}"]

[output]
prefix = "fifo_fastq"
"#,
        sanitize_path(&read1),
        sanitize_path(&read2)
    );
    fs::write(&config_path, config).expect("write config");

    let (tx, rx) = mpsc::channel();
    let fifo_reader_path = fifo_path.clone();
    thread::spawn(move || {
        let mut reader = fs::File::open(&fifo_reader_path).expect("open fifo for read");
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).expect("read fifo");
        tx.send(buffer).expect("send fifo data");
    });

    run(&config_path, &output_dir, false).expect("run processor");

    let data = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("receive fifo data");
    let contents = String::from_utf8(data).expect("fifo utf8");
    let read_count = contents
        .lines()
        .filter(|line| line.starts_with('@'))
        .count();
    assert_eq!(read_count, READ_COUNT);
}

#[cfg(unix)]
#[test]
fn chunked_output_rejected_for_named_pipe() {
    let (_temp, read1, read2, config_path, output_dir) = common_setup();
    let first_chunk_fifo = output_dir.join("chunked_fastq_read1.0000.fq");
    create_fifo(&first_chunk_fifo).expect("create fifo");

    let config = format!(
        r#"[input]
read1 = ["{}"]
read2 = ["{}"]

[output]
prefix = "chunked_fastq"
Chunksize = 3
"#,
        sanitize_path(&read1),
        sanitize_path(&read2)
    );
    fs::write(&config_path, config).expect("write config");

    let error = run(&config_path, &output_dir, false).expect_err("chunking should fail for fifo");
    let message = format!("{error:#}");
    assert!(
        message.contains("named pipe"),
        "unexpected error message: {message}"
    );
}
