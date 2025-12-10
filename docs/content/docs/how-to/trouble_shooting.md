---
weight: 1
---

# Troubleshooting


## I don't know what to do after an error message


Well, that's below our targets, we want our error messages to tell
you enough for you to be able to fix it.

Please file a bug report in our [issue tracker](https://github.com/tyberiusPrime/mbf-fastq-processor/issues).

## I received a friendly panic message

These look like this

```bash
Well, this is embarrassing.

mbf-fastq-processor had a problem and crashed. To help us diagnose the problem you can send us a crash report.

We have generated a report file at "/tmp/nix-shell.ty6dWk/report-89ffc0f3-8076-4f55-83f0-c0a5d2fb3b55.toml".
```

This kind of error message, which wraps a rust 'panic' only happens if mbf-fastq-processor has managed
to get into an impossible or unforeseen state.

That's always a bug (even on invalid input, the issue should be trapped by non-panic code).

Please review the temp file it's created and [attach it to an issue](https://github.com/tyberiusPrime/mbf-fastq-processor/issues/new).

Ideally of course with a minimal data set that reproduces the crash,
but just the traceback will tell us quite a bit.


## The output isn't what was expected

Please prepare a minimal example for us to run, and what output you'd expect
and [submit an
issue](https://github.com/tyberiusPrime/mbf-fastq-processor/issues/new).

We'll then see if it's a bug, or a documentation issue.

