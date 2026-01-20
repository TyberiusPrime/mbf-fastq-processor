#!/usr/bin/env bash
sed -i 's#^1 = .*#1 = "input_read_1.fq"#' config.toml
printf '@read1\nAGTC\n+\nIIHI\n' > input_read_1.fq
cat config.toml
