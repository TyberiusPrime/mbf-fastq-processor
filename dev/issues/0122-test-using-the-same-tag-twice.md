status: closed
# Test using the same tag twice
Error? Ok if we delete in between?

Added test cases:
1. `test_cases/error_handling/duplicate_tag_name/` - Same tag twice without ForgetTag (error)
2. `test_cases/single_step/extraction/use_tag_after_forget/` - Same tag twice with ForgetTag in between (ok)
