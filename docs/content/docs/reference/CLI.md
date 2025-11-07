---
weight: 2
not-a-transformation: true
---
# Command line interface

mbf-fastq-processor is configured exclusively through a TOML document. The CLI is therefore intentionally minimal and focuses on selecting the configuration and the working directory.

## Usage

```text
mbf-fastq-processor process <config.toml>  [--allow-overwrite]
mbf-fastq-processor template
mbf-fastq-processor wizard
```


### Process

Process FASTQ as described in <config.toml>.(see the [TOML format reference]({{< relref "docs/reference/toml" >}})). Relative paths are resolved against the current shell directory.

#### (not) "Done" marker file
By default, existence of any output file will lead to an early abort, 
before any processing happens (other output files might have been created with 0 bytes at this point though). If you pass --allow-overwrite (or if an output.incomplete file exists), existing output files are overwritten instead.

The output.incomplete file exists until the successful exit of mbf-fastq-processor.
This way you can detect incomplete runs by the existence of that file.


#### Behaviour

- Exit status `0` denotes success; non-zero exit codes indicate configuration, IO, or data validation failures.
- Error messages go to stderr. Helpful hints for configuration issues also go to stderr.
- --help goes to stdout. Run without arguments shows help, going to stderr.
- by default there is no stdout output. [Progress]({{< relref "docs/reference/report-steps/Progress.md" >}}) can change that. 


### Template
Output a configuration file showing all the options, ready to be 'uncommented'.

The template is also available [here](../toml/template.toml).

Appropriate parts of the template are also shown when a configuration error is detected.


### Wizard

We have a configuration generating/editing TUI wizard reachable via `mbf-fastq-processor wizard`.

## Development with Cargo

During development you can run the CLI straight from the workspace:

```bash
cargo run --release -- process path/to/config.toml
```

If your configuration only defines `[input]` and `[output]`, the processor behaves much like a decompression-aware `cat`, copying reads while honouring the configured compression format.
