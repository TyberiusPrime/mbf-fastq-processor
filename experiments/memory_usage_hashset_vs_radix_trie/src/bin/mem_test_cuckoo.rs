use rand_chacha::rand_core::SeedableRng;
use scalable_cuckoo_filter::ScalableCuckooFilter;
use std::collections::HashSet;
use std::io::BufRead;

extern crate stats_alloc;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

type OurCuckCooFilter = scalable_cuckoo_filter::ScalableCuckooFilter<
    [u8],
    scalable_cuckoo_filter::DefaultHasher,
    rand_chacha::ChaChaRng,
>;
fn extend_seed(seed: u64) -> [u8; 32] {
    let seed_bytes = seed.to_le_bytes();

    // Extend the seed_bytes to 32 bytes
    let mut extended_seed = [0u8; 32];
    extended_seed[..8].copy_from_slice(&seed_bytes);
    extended_seed
}

fn main() {
    let mut reg = Region::new(&GLOBAL);
    let mut total = 0;
    let start = std::time::Instant::now();
    let filename = std::env::args().nth(1).unwrap();
    let handle = std::fs::File::open(filename).unwrap();
    let seed = 42;
    let rng = rand_chacha::ChaChaRng::from_seed(extend_seed(seed));
    let mut set: OurCuckCooFilter = scalable_cuckoo_filter::ScalableCuckooFilterBuilder::new()
        .initial_capacity(1_000)
        .false_positive_probability(0.000001)
        .rng(rng)
        .finish();

    for line in std::io::BufReader::new(handle).lines().step_by(4) {
        let line = line.unwrap();
        //println!("{line}");
        total += line.len();
        set.insert(line.as_bytes());
    }
    println!("Stats at 1: {:#?}", reg.change());
    let set2 = set.clone();
    println!("Stats at 2: {:#?}", reg.change_and_reset());
    let set3 = set2.clone();
    println!("Stats at 3: {:#?}", reg.change_and_reset());

    println!("Number of unique reads: {}", set3.len());

    let stop = start.elapsed();
    println!("elapsed: {:?}", stop);
    println!("raw bytes {total}");
}
