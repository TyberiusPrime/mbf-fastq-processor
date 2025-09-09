Introduce a ValidateName step, that validates
that all the names (across read1/read2/index1/index2 as present)
are either identical (if strict=true)
or share a common prefix up to the first `name_comment_separator`
(only must be set if strict = false).

add test cases.
