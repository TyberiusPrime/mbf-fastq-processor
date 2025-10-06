---
weight: 2
not-a-transformation: true
---
# Command line interface

mbf-fastq-processor is configured exclusively through a TOML document. The CLI is therefore intentionally minimal and focuses on selecting the configuration and the working directory.

## Usage

```text
mbf-fastq-processor <config.toml> [working_directory]
```

- `config.toml` – required path to the pipeline description (see the [TOML format reference]({{< relref "docs/reference/toml.md" >}})). Relative paths are resolved against the current shell directory.
- `working_directory` – optional override for the process working directory. Use it to anchor relative paths inside the configuration without changing your shell location.

When `working_directory` is omitted, the binary inherits the environment's current directory.

## Behaviour

- Exit status `0` denotes success; non-zero exit codes indicate configuration, IO, or data validation failures.
- Log output is emitted on stderr. Enable structured logging with `RUST_LOG=info` (or `debug`) when troubleshooting.
- The process honours OS-level resource limits and can be composed in scripts or pipelines; the FastQ stream on stdout is available when `[output]` sets `stdout = true`.

## Working with Cargo

During development you can run the CLI straight from the workspace:

```bash
cargo run --release -- path/to/config.toml
```

Cargo passes `--` through to the binary; add the optional working directory argument if needed. For faster iteration without full optimisation use `cargo run --`.

If your configuration only defines `[input]` and `[output]`, the processor behaves much like a decompression-aware `cat`, copying reads while honouring the configured compression format.
