use std::collections::HashSet;
use std::io::BufRead;


extern crate stats_alloc;

use stats_alloc::{StatsAlloc, Region, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
    let start = std::time::Instant::now();
    let filename = std::env::args().nth(1).unwrap();
    let mut set = HashSet::new();
    let handle = ex::fs::File::open(filename).unwrap();
    let mut reg = Region::new(&GLOBAL);
    let mut total = 0;
    for line in std::io::BufReader::new(handle).lines().step_by(4) {
        let line = line.unwrap();
        //println!("{line}");
        total += line.len();
        set.insert(line);
    }
    println!("Stats at 1: {:#?}", reg.change());
    let set2 = set.clone();
    println!("Stats at 2: {:#?}", reg.change_and_reset());
    let set3 = set2.clone();
    println!("Stats at 2: {:#?}", reg.change_and_reset());

    println!("Number of unique reads: {}", set3.len());

    let stop = start.elapsed();
    println!("elapsed: {:?}", stop);
    println!("raw bytes {total}");
}