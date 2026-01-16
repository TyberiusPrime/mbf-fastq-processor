status: closed
# head still doesn't always work.

Possibly because of report?
```
[input]
read1 = ["read1.fq.gz", "read2.fq.gz""]

[output]
prefix = "transformed"
format = "Raw"
output_hash_compressed = true
output = ["read1", "read2"]
report_json = true
report_html = true


[[step]]
	action = 'Head'
	n = 100_000

[[step]]
action = "Report"
label = "report.before"
count = true
base_statistics = true
length_distribution = true
duplicate_count_per_read = true

[[step]]
action = "TagOtherFileByName"
segment = "read1"
label = "in_zea"
filename = "some_bam_file"
false_positive_rate = 0
seed = 42
ignore_unaligned = true
readname_end_chars = " "


[[step]]
	action = "FilterByBoolTag"
	label = "in_zea"
	keep_or_remove = 'remove'

[[step]]
action = "Report"
label = "report.after"
count = true
base_statistics = true
length_distribution = true
duplicate_count_per_read = true
```

--
I think I got this tested out now.
