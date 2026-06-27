use allocation_counter::measure;
use fastqrab::cli::main::{EarlyExit, entry_point};

fn main() {
    let mut result: Result<(), anyhow::Error> = Ok(());
    let info = measure(|| {
        result = entry_point();
    });
    eprintln!(
        "alloc: count_total={} count_max={} count_current={} bytes_total={} bytes_max={} bytes_current={}",
        info.count_total,
        info.count_max,
        info.count_current,
        info.bytes_total,
        info.bytes_max,
        info.bytes_current
    );
    match result {
        Ok(()) => {}
        Err(e) if e.is::<EarlyExit>() => std::process::exit(1),
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
