status: open
# Mutational bugs

MISSED   src/transformations/edits/swap_conditional.rs:99:58: replace == with != in <impl Step for SwapConditional>::apply in 8.1s build + 6.8s test

-> Add / change Test case SwapConditional
(and swap) where hit region is in 2nd segment


MISSED   src/io/output.rs:35:26: replace == with != in write_read_to_bam in 8.2s build + 6.8s test

add pe bam output test with interleaved data, verify segmented flas.

