#!/bin/bash

# Create an existing output file to trigger the error
echo "@existing_output" > output_read1.fq
echo "GATCGATC" >> output_read1.fq  
echo "+" >> output_read1.fq
echo "AAAAAAAA" >> output_read1.fq
chmod 000 output_read1.fq
