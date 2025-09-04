#!/bin/bash

# Create an existing output file to trigger the error
echo "@existing_output" > output_1.fq
echo "GATCGATC" >> output_1.fq  
echo "+" >> output_1.fq
echo "AAAAAAAA" >> output_1.fq
chmod 000 output_1.fq
