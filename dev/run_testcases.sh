#/usr/bin/bash

fd "panic|\.rs\$|\.toml\$|sha256" | grep -v actual | entr cargo run --release --bin mbf-fastq-processor-test-runner test_cases/integration_tests/
