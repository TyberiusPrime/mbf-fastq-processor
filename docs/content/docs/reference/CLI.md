---
weight: 2
not-a-transformation: true
---
# Command line interface

mbf-fastq-processor is configured exclusively through a TOML document. The CLI is therefore intentionally minimal and focuses on selecting the configuration and the working directory.

## Usage

```text
mbf-fastq-processor process <config.toml> 
mbf-fastq-processor template
```


### Process

Process FastQ as described in <config.toml>.(see the [TOML format reference]({{< relref "docs/reference/toml" >}})). Relative paths are resolved against the current shell directory.


### Template
Output a configuration file showing all the options, ready to be 'uncommented'.

The template is also available [here](../toml/template.toml).


## Behaviour

- Exit status `0` denotes success; non-zero exit codes indicate configuration, IO, or data validation failures.

## Working with Cargo

During development you can run the CLI straight from the workspace:

```bash
cargo run --release -- process path/to/config.toml
```

If your configuration only defines `[input]` and `[output]`, the processor behaves much like a decompression-aware `cat`, copying reads while honouring the configured compression format.
