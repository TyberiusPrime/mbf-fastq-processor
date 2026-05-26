use anyhow::Result;

use fastqrab::cli::main::entry_point;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
    entry_point()
}
