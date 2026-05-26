use allocation_counter::measure;
use anyhow::Result;
use fastqrab::cli::main::entry_point;

fn main() -> Result<()> {
    let info = measure(|| {
        let _ = entry_point();
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
    entry_point()
}
