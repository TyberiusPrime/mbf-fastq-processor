use allocation_counter::measure;
use fastqrab::cli::main::{EarlyExit, entry_point};

fn main() {
    let mut result: Result<(), anyhow::Error> = Ok(());
    let info = measure(|| {
        result = entry_point();
    });
    // The decompressor runs as a subprocess of this same binary (`fastqrab
    // __decompressor`), so it would otherwise print its own `alloc:` line. The
    // test harness expects exactly one alloc line per invocation, so suppress
    // the print in the subprocess and only report the parent's accounting.
    if !std::env::args().any(|a| a == "__decompressor") {
        eprintln!(
            "alloc: count_total={} count_max={} count_current={} bytes_total={} bytes_max={} bytes_current={}",
            info.count_total,
            info.count_max,
            info.count_current,
            info.bytes_total,
            info.bytes_max,
            info.bytes_current
        );
    }
    match result // cov:excl-line
    {
        // why though?, the ok is being hit.
        Ok(()) => {}
        //cov:excl-start - the test only covers the working case, and that's ok.
        Err(e) if e.is::<EarlyExit>() => std::process::exit(1),
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        } //cov:excl-stop
    }
}
