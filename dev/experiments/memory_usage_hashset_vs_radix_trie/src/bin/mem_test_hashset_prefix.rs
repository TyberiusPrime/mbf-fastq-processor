use std::collections::HashSet;
use std::io::BufRead;

extern crate stats_alloc;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

pub fn longest_common_prefix(strs: &Vec<String>) -> String {
    if strs.is_empty() {
        return String::new();
    }

    let mut prefix = strs[0].clone();

    for s in strs.iter() {
        while !s.starts_with(&prefix) {
            if prefix.is_empty() {
                return String::new();
            }
            prefix.pop(); // Shorten the prefix
        }
    }

    prefix
}

fn find_prefix(filename: &str) -> String {
    let handle = ex::fs::File::open(filename).unwrap();
    let mut first = Vec::new();
    let iter = std::io::BufReader::new(handle).lines().step_by(4);
    for line in iter.take(100) {
        let line = line.unwrap();
        first.push(line);
    }
    longest_common_prefix(&first)
}

fn build_set(filename: &str, prefix: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    let handle = ex::fs::File::open(&filename).unwrap();
    let iter = std::io::BufReader::new(handle).lines().step_by(4);

    let mut total =0;
    for line in iter {
        let line = line.unwrap();
        let s = line.trim_start_matches(&prefix).to_string();
        total += s.len();
        set.insert(s);
    }
    println!("total bytes {total}");
    set
}

fn main() {
    let start = std::time::Instant::now();
    let filename = std::env::args().nth(1).unwrap();
    let mut reg = Region::new(&GLOBAL);

    println!("Stats at 0: {:#?}", reg.change());
    let prefix = find_prefix(&filename);
    let set = build_set(&filename, &prefix);
    println!("Stats at 1: {:#?}", reg.change_and_reset());
    let set2 = set.clone();
    println!("Stats at 2: {:#?}", reg.change_and_reset());
    let set3 = set2.clone();
    println!("Stats at 3: {:#?}", reg.change_and_reset());

    println!("Number of unique reads: {}", set3.len());

    let stop = start.elapsed();
    println!("elapsed: {:?}", stop);
}