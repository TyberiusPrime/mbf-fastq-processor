use std::io::BufRead;

extern crate stats_alloc;

use stats_alloc::{StatsAlloc, Region, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;
fn main() {
    let mut reg = Region::new(&GLOBAL);
    let start = std::time::Instant::now();
    let filename = std::env::args().nth(1).unwrap();
    let mut set = radix_trie::Trie::<String, ()>::new();
    let handle = ex::fs::File::open(filename).unwrap();
    for line in std::io::BufReader::new(handle).lines().step_by(4) {
        let line = line.unwrap();
        //println!("{line}");
        set.insert(line, ());
    }
    println!("Stats at 1: {:#?}", reg.change_and_reset());
    let set2 = set.clone();
    println!("Stats at 2: {:#?}", reg.change());

    println!("query: {:?}", set2.get("shU"));
    let stop = start.elapsed();
    println!("elapsed: {:?}", stop);
}