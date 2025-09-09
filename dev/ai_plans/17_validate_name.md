Introduce a ValidateName step, that validates
that all the names (across read1/read2/index1/index2 as present)
are either identical 
or share a common prefix up to the first `readname_end_chars`
(optional field)

add test cases.
