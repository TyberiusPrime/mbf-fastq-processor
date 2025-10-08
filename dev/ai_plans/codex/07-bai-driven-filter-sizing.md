# this one worked but for minor details
# BAI-driven filter sizing

1. Add shared helper that inspects BAM index to estimate cuckoo filter capacity.
2. Use the helper from both `other_file_by_name` and `other_file_by_sequence` initializers.
3. Run targeted integration tests that cover indexed BAM input.
