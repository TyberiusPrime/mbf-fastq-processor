use fastqrab::cli::main::{EarlyExit, entry_point};

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    match entry_point() {
        Ok(()) => {}
        Err(e) if e.is::<EarlyExit>() => std::process::exit(1), //mutants::skip
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
