sed -i 's#^1 = .*#1 = "input_read_1.fq"#' config.toml
echo '@read1' > input_read_1.fq
echo 'AGTC' >> input_read_1.fq
echo '+' >> input_read_1.fq
echo 'IIHI' >> input_read_1.fq
cat config.toml
