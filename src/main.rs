use std::path::PathBuf;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let toml_file = std::env::args()
        .nth(1)
        .context("First argument must be a toml file path.")?;
    let toml_file = PathBuf::from(toml_file);
    let current_dir = std::env::args()
        .nth(2)
        .map(|x| PathBuf::from(x))
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    mbf_fastq_processor::run(&toml_file, &current_dir)
}

/* fn main() -> Result<()> {
    let filename = "benchmarks/data/large/ERR12828869_1.fq.zst";
    let mut fh = mbf_fastq_processor::io::open_file(filename).unwrap();
    let mut last_partial = mbf_fastq_processor::io::PartialStatus::NoPartial;
    let mut last_partial_read = None;
    let mut total = 0;
    let start_time = std::time::Instant::now();
    loop {
        let mut block = vec![0u8; 1024 *1024 * 10];
        let more = fh.read(&mut block).expect("Could not read block");
        //println!("read block {last_partial:?}, size: {more}", );
        block.resize(more, 0); //restrict to actually read bytes
        if more == 0 {
            break;
        }
        let fq_block =
            mbf_fastq_processor::io::parse_to_fastq_block(block, last_partial, last_partial_read)
                .unwrap();
        last_partial = fq_block.status;
        last_partial_read = fq_block.partial_read;
        total += fq_block.block.entries.len();
        //blocks.push(fq_block.block);
    }
    println!("Total entries: {total}");
    let stop_time = std::time::Instant::now();
    println!("Time: {:.3}", (stop_time - start_time).as_secs_f64());

    Ok(())
} */

/* fn main() -> Result<()> {
    use std::io::ErrorKind;
    let filename = "benchmarks/data/large/ERR12828869_1.fq";
    let mut count = 0;
    let mut total = 0usize;

    let fh = std::fs::File::open(filename).context(format!("Could not open file {}", filename))?;
    let mut reader = std::io::BufReader::new(fh);
    loop {
        let mut line1 = Vec::new();
        let mut line2 = Vec::new();
        let mut line3 = Vec::new();
        let mut line4 = Vec::new();
        let mut dummy: [u8; 1] = [0];
        match reader.read_exact(&mut dummy) {
            Ok(()) => (),
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => {
                    println!("count: {} {total}", count);
                    return Ok(());
                }
                _ => panic!("Problem reading fastq"),
            },
        }
        if dummy[0] != b'@' {}
        let more = reader
            .read_until(b'\n', &mut line1)
            .expect("Could not read line 1.");
        if more == 0 {
            panic!("File truncated");
        }
        reader
            .read_until(b'\n', &mut line2)
            .expect("Could not read line 2.");
        reader
            .read_until(b'\n', &mut line3) //we don't care about that one'
            .expect("Could not read line.");
        reader
            .read_until(b'\n', &mut line4)
            .expect("Could not read line 4.");
        line1.pop();
        line2.pop();
        line4.pop();

        if line2.len() != line4.len() {
            dbg!(&std::str::from_utf8(&line1));
            dbg!(&std::str::from_utf8(&line2));
            dbg!(&std::str::from_utf8(&line3));
            dbg!(&std::str::from_utf8(&line4));
            panic!("Truncated fastq file")
        }
        total += line2.len();
        total += line1.len();
        total += line4.len();
        count += 1;
    }
} */
