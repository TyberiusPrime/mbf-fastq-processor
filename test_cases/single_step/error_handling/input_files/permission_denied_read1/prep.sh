#!/usr/bin/env bash
sed -i 's#^read1 = .*#read1 = "input_read1.fq"#' config.toml
touch input_read1.fq
chmod 000 input_read1.fq
