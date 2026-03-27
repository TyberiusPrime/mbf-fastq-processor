use std::cell::Cell;
use std::collections::HashSet;
use std::time::Instant;

use allocation_counter::{measure, AllocationInfo};
use anyhow::{ensure, Context, Result};
use bstr::BString;
use clap::{Args, Parser, Subcommand};
use scalable_bloom_filter::ScalableBloomFilter;
use scalable_cuckoo_filter::ScalableCuckooFilter;

const PAYLOAD_LEN: usize = 150;
const VALUE_BYTES: usize = 8;
const DEFAULT_CAPACITY: u64 = 100_000;

#[derive(Parser)]
#[command(author, version, about = "Deduplication filter experiments", long_about = None)]
struct Cli {
    /// Choose which data structure to benchmark
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Benchmark the scalable cuckoo filter
    Cuckoo(FilterArgs),
    /// Benchmark the scalable bloom filter
    Bloom(FilterArgs),
    /// Benchmark a standard HashSet of BStrings for comparison
    Hashset(FilterArgs),
    /// Run all benchmarks sequentially with the same parameters
    All(FilterArgs),
}

#[derive(Args, Clone)]
struct FilterArgs {
    /// Number of elements to insert
    #[arg(long, default_value_t = 1_000_000_000, value_parser = clap::value_parser!(u64).range(1..))]
    elements: u64,
    /// Initial capacity for the data structure (defaults to `elements`)
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
    initial_capacity: Option<u64>,
    /// Target false positive probability for scalable filters
    #[arg(long = "false-positive-probability", default_value_t = 0.001)]
    false_positive_probability: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Cuckoo(args) => {
            let report = run_cuckoo(&args)?;
            print_report(&report);
        }
        Command::Bloom(args) => {
            let report = run_bloom(&args)?;
            print_report(&report);
        }
        Command::Hashset(args) => {
            let report = run_hashset(&args)?;
            print_report(&report);
        }
        Command::All(args) => {
            let cuckoo = run_cuckoo(&args)?;
            let bloom = run_bloom(&args)?;
            let hashset = run_hashset(&args)?;
            for report in [cuckoo, bloom, hashset] {
                print_report(&report);
                println!();
            }
        }
    }
    Ok(())
}

fn run_cuckoo(args: &FilterArgs) -> Result<RunReport> {
    ensure!(
        (0.0..1.0).contains(&args.false_positive_probability),
        "false positive probability must be in (0, 1)"
    );
    let elements = args.elements;
    let initial_capacity_u64 = args.initial_capacity.unwrap_or(100_000);
    let initial_capacity = usize::try_from(initial_capacity_u64).with_context(|| {
        format!("initial capacity {initial_capacity_u64} does not fit in usize")
    })?;

    let false_positives = Cell::new(0u64);
    let bits = Cell::new(0u64);
    let capacity = Cell::new(0usize);
    let len = Cell::new(0usize);
    let effective_fpp = Cell::new(args.false_positive_probability);

    let timer = Instant::now();
    // allocation_counter::measure wraps the workload so we can inspect allocation stats afterwards.
    let allocation = measure(|| {
        let mut filter = ScalableCuckooFilter::<[u8; PAYLOAD_LEN]>::new(
            initial_capacity,
            args.false_positive_probability,
        );

        let mut payload = [0u8; PAYLOAD_LEN];
        for value in 0..elements {
            write_payload(&mut payload, value);
            if filter.contains(&payload) {
                false_positives.set(false_positives.get() + 1);
            } else {
                filter.insert(&payload);
            }
        }

        bits.set(filter.bits());
        capacity.set(filter.capacity());
        len.set(filter.len());
        effective_fpp.set(filter.false_positive_probability());
    });
    let elapsed = timer.elapsed();

    Ok(RunReport {
        name: "Scalable Cuckoo Filter",
        inserted: elements,
        elapsed,
        allocation,
        false_positives: false_positives.get(),
        extra: Extra::Cuckoo {
            bits: bits.get(),
            capacity: capacity.get(),
            len: len.get(),
            configured_fpp: args.false_positive_probability,
            effective_fpp: effective_fpp.get(),
        },
    })
}

fn run_bloom(args: &FilterArgs) -> Result<RunReport> {
    ensure!(
        (0.0..1.0).contains(&args.false_positive_probability),
        "false positive probability must be in (0, 1)"
    );
    let elements = args.elements;
    let initial_capacity_u64 = args.initial_capacity.unwrap_or(DEFAULT_CAPACITY);

    let initial_capacity = usize::try_from(initial_capacity_u64).with_context(|| {
        format!("initial capacity {initial_capacity_u64} does not fit in usize")
    })?;

    let false_positives = Cell::new(0u64);
    let allocated_bits = Cell::new(0usize);
    let used_bits = Cell::new(0usize);

    let timer = Instant::now();
    // allocation_counter::measure wraps the workload so we can inspect allocation stats afterwards.
    let allocation = measure(|| {
        let mut filter = ScalableBloomFilter::<[u8; PAYLOAD_LEN]>::new(
            initial_capacity,
            args.false_positive_probability,
        );

        let mut payload = [0u8; PAYLOAD_LEN];
        for value in 0..elements {
            write_payload(&mut payload, value);
            if filter.contains(&payload) {
                false_positives.set(false_positives.get() + 1);
            } else {
                filter.insert(&payload);
            }
        }

        allocated_bits.set(filter.allocated_bits());
        used_bits.set(filter.used_bits());
    });
    let elapsed = timer.elapsed();

    Ok(RunReport {
        name: "Scalable Bloom Filter",
        inserted: elements,
        elapsed,
        allocation,
        false_positives: false_positives.get(),
        extra: Extra::Bloom {
            allocated_bits: allocated_bits.get(),
            used_bits: used_bits.get(),
        },
    })
}

fn run_hashset(args: &FilterArgs) -> Result<RunReport> {
    let elements = args.elements;
    let initial_capacity_u64 = args.initial_capacity.unwrap_or(DEFAULT_CAPACITY);

    let capacity_hint = usize::try_from(initial_capacity_u64).with_context(|| {
        format!("initial capacity {initial_capacity_u64} does not fit in usize")
    })?;

    let false_positives = Cell::new(0u64);
    let final_capacity = Cell::new(0usize);

    let timer = Instant::now();
    // allocation_counter::measure wraps the workload so we can inspect allocation stats afterwards.
    let allocation = measure(|| {
        let mut set: HashSet<BString> = HashSet::with_capacity(capacity_hint);

        for value in 0..elements {
            let bytes = BString::from(payload_vec(value));
            if set.contains(&bytes) {
                false_positives.set(false_positives.get() + 1);
            }
            set.insert(bytes);
        }

        final_capacity.set(set.capacity());
    });
    let elapsed = timer.elapsed();

    Ok(RunReport {
        name: "HashSet<BString>",
        inserted: elements,
        elapsed,
        allocation,
        false_positives: false_positives.get(),
        extra: Extra::HashSet {
            capacity: final_capacity.get(),
        },
    })
}

fn write_payload(buffer: &mut [u8; PAYLOAD_LEN], value: u64) {
    buffer[..VALUE_BYTES].copy_from_slice(&value.to_le_bytes());
    buffer[VALUE_BYTES..].fill(0);
}

fn payload_vec(value: u64) -> Vec<u8> {
    let mut data = vec![0u8; PAYLOAD_LEN];
    data[..VALUE_BYTES].copy_from_slice(&value.to_le_bytes());
    data
}

struct RunReport {
    name: &'static str,
    inserted: u64,
    elapsed: std::time::Duration,
    allocation: AllocationInfo,
    false_positives: u64,
    extra: Extra,
}

enum Extra {
    Cuckoo {
        bits: u64,
        capacity: usize,
        len: usize,
        configured_fpp: f64,
        effective_fpp: f64,
    },
    Bloom {
        allocated_bits: usize,
        used_bits: usize,
    },
    HashSet {
        capacity: usize,
    },
}

fn print_report(report: &RunReport) {
    let duration_secs = report.elapsed.as_secs_f64();
    let ops = if duration_secs > 0.0 {
        (report.inserted) as f64 / duration_secs
    } else {
        0.0
    };
    let queries = report.inserted;
    let fpr = if queries > 0 {
        report.false_positives as f64 / queries as f64
    } else {
        0.0
    };

    println!("{}", report.name);
    println!("  inserted: {}", report.inserted);
    println!("  duration: {:.3}s (≈{:.3} ops/s)", duration_secs, ops);

    let total_mib = report.allocation.bytes_total as f64 / 1024.0 / 1024.0;
    let peak_mib = report.allocation.bytes_max as f64 / 1024.0 / 1024.0;
    let leaks_mib = report.allocation.bytes_current as f64 / 1024.0 / 1024.0;

    println!(
        "  allocations: {{ total: {} ({} bytes ≈{:.3} MiB), peak: {} ({} bytes ≈{:.3} MiB) }}",
        report.allocation.count_total,
        report.allocation.bytes_total,
        total_mib,
        report.allocation.count_max,
        report.allocation.bytes_max,
        peak_mib
    );
    println!(
        "  leaks (net allocations): {} ({} bytes ≈{:.3} MiB)",
        report.allocation.count_current, report.allocation.bytes_current, leaks_mib
    );
    println!(
        "  false positives: {} (rate {:.6})",
        report.false_positives, fpr
    );

    match &report.extra {
        Extra::Cuckoo {
            bits,
            capacity,
            len,
            configured_fpp,
            effective_fpp,
        } => {
            println!(
                "  filter bits: {} (~{:.3} MiB)",
                *bits,
                *bits as f64 / 8.0 / 1024.0 / 1024.0
            );
            println!("  filter capacity: {}", *capacity);
            println!("  filter len: {}", *len);
            println!(
                "  configured/actual FPP: {:.6} / {:.6}",
                configured_fpp, effective_fpp
            );
        }
        Extra::Bloom {
            allocated_bits,
            used_bits,
        } => {
            let alloc_mib = *allocated_bits as f64 / 8.0 / 1024.0 / 1024.0;
            let used_mib = *used_bits as f64 / 8.0 / 1024.0 / 1024.0;
            println!(
                "  allocated bits: {} (~{:.3} MiB)",
                *allocated_bits, alloc_mib
            );
            println!("  used bits: {} (~{:.3} MiB)", *used_bits, used_mib);
        }
        Extra::HashSet { capacity } => {
            println!("  hashset capacity: {}", *capacity);
        }
    }
}
