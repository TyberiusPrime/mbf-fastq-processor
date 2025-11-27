---
weight: 2
not-a-transformation: true
---
# Command line interface

mbf-fastq-processor is configured exclusively through a TOML document. The CLI is therefore intentionally minimal and focuses on selecting the configuration and the working directory.

## Usage

```text
mbf-fastq-processor process [config.toml]  [--allow-overwrite]
mbf-fastq-processor template
mbf-fastq-processor verify [config.toml] [--output-dir <OUTPUT_DIR>]
mbf-fastq-processor interactive [config.toml]
mbf-fastq-processor completions <SHELL>
```


### Process

Process FASTQ as described in <config.toml>.(see the [TOML format reference]({{< relref "docs/reference/toml" >}})). 
Relative paths are resolved against the current shell directory.

The config.toml argument can be left off iff there's one .toml in the current directory, and it contains an [input] and an [output] section



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


### Verify

The verify command runs processing in a temporary directory and compares the outputs against expected outputs in the same directory as the configuration file.

This is useful for:
- Testing that your pipeline produces expected results
- Regression testing during development
- Validating that changes don't affect output

#### Usage

```bash
mbf-fastq-processor verify [config.toml] [--output-dir <OUTPUT_DIR>]
```

If no configuration file is specified, the tool will auto-detect a single .toml 
file in the current directory if that file contains both `[input]` and `[output]` sections.

#### Behavior

- Runs processing in a temporary directory with absolute input paths
- Compares all output files (matching the output prefix) against expected files in the config directory
- If files called 'stdout' or 'stderr' exist, compare these to actual stdout/stderr 
- If configuration uses stdin (`--stdin--`) and a file named 'stdin' exists in the config directory, pipes that file's content to the subprocess as stdin
- Normalizes dynamic content in JSON/HTML reports (timestamps, paths, versions) before comparison
- Reports missing files, unexpected files, and content mismatches
- Exit code 0 indicates successful verification; non-zero indicates failure

#### Output Directory Option

The `--output-dir` option specifies a directory to copy the actual outputs to if verification fails:

```bash
mbf-fastq-processor verify config.toml --output-dir ./debug_output
```

When verification fails with this option:
1. The specified directory is removed if it exists (!)
2. All output files from the temporary directory are copied to it
3. JSON and HTML files are normalized (same normalization used for comparison)
4. stdout/sterr are logged into files `stdout` and `stderr`
4. This makes it easy to inspect what the processor actually generated

This is particularly useful for:
- Debugging test failures
- Updating expected outputs after intentional changes
- Understanding differences between expected and actual results

#### Stdin Support

When your configuration uses stdin input (by specifying `--stdin--` as an input file), the verify command can simulate stdin input by reading from a file named `stdin` in the same directory as your configuration file.

For example, if your `config.toml` contains:
```toml
[input]
read1 = '--stdin--'
```

And you have a file named `stdin` in the same directory, the verify command will pipe that file's content to the subprocess as stdin input, allowing you to test stdin-based processing in a reproducible way.

### Interactive

The interactive mode takes your configuration file,
samples 15 from the first 10,000 reads (configurable via CLI arguments),
and shows you the [Inspect]({{< relref "docs/reference/report-steps/Inspect.md" >}}) results.

Every time you save, the results refresh.

This way you can quickly tune and work on your configuration.

### Completions

Generate shell completion scripts for command-line auto-completion in various shells.

#### Supported Shells

- **bash** - Bourne Again Shell
- **fish** - Friendly Interactive Shell
- **zsh** - Z Shell
- **powershell** - PowerShell
- **elvish** - Elvish Shell

#### Installation Instructions

**Bash**

Add to `~/.bashrc`:
```bash
source <(mbf-fastq-processor completions bash)
```

Or for environment-based approach (auto-updates):
```bash
eval "$(COMPLETE=bash mbf-fastq-processor)"
```

**Fish**

Save to Fish completions directory:
```fish
mbf-fastq-processor completions fish > ~/.config/fish/completions/mbf-fastq-processor.fish
```

Or add to `~/.config/fish/config.fish` for environment-based approach:
```fish
if command -v mbf-fastq-processor > /dev/null
    COMPLETE=fish mbf-fastq-processor | source
end
```

**Zsh**

Add to `~/.zshrc`:
```zsh
source <(mbf-fastq-processor completions zsh)
```

Or for environment-based approach:
```zsh
eval "$(COMPLETE=zsh mbf-fastq-processor)"
```

**PowerShell**

Add to your PowerShell profile:
```powershell
mbf-fastq-processor completions powershell | Out-String | Invoke-Expression
```

#### Features

Shell completions provide:
- Command and subcommand completion
- File path completion for configuration files
- Directory path completion for output directories
- Shell-specific syntax and behavior

After installing completions, restart your shell or source the configuration file for changes to take effect.

## Development with Cargo

During development you can run the CLI straight from the workspace:

```bash
cargo run --release -- process path/to/config.toml
```

If your configuration only defines `[input]` and `[output]`, the processor behaves much like a decompression-aware `cat`, copying reads while honouring the configured compression format.
