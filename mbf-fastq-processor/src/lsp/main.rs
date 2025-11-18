//! Language Server Protocol implementation for mbf-fastq-processor
//!
//! This LSP server provides auto-completion, validation, and documentation
//! for mbf-fastq-processor TOML configuration files.

mod backend;
mod completion;
mod diagnostics;
mod hover;

use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Set up logging to stderr (LSP communication happens on stdin/stdout)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Starting mbf-fastq-processor language server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| backend::Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
