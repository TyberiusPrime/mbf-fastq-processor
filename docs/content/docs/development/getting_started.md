# Getting started with development

The easiest way to get started with working on mbf-fastq-processor
is to clone the repository:

```bash
jj git clone --colocate https://github.com/TyberiusPrime/mbf-fastq-processor
```

or  
```bash 
git clone  https://github.com/TyberiusPrime/mbf-fastq-processor
```
if you're not yet convinced that [Jujutsu](https://docs.jj-vcs.dev/latest/) is the better git.

## Development environment

Using `nix develop` to enter a shell with all the necessary requirements using [Nix](https://nix.dev/).

If you don't use nix, you're on your own to supply a matching rust compiler, 
openssl and pkg-config.


## Development with Cargo

During development you can run the CLI straight from the workspace:

```bash
cargo run --release -- process path/to/config.toml
```

--release is optional, but cargo debug builds can be slow.

See the [CLI]({{< relref "docs/reference/CLI.md" >}}) documentation for arguments after 
the `--` that splits the cargo run arguments from the process arguments.

## Running tests

```bash
cargo test # runs all tests
```

```bash
cargo test <test-name-substring> # runs specifc tests
```

## Benchmarking
``bash 
cargo bench 
``

will get you a (not so) quick benchmark of our individual components.


## Manipulating core count

In addition to the config options in the
[input-section]({{< relref "docs/reference/input-section.md" >}})
or in the [Options section]({{< relref "docs/reference/Options.md" >}}),
you can influence the number of cores used for the automatic decisions on 
thread counts by setting the `MBF_FASTQ_PROCESSOR_NUM_CPUS` environment variable.
