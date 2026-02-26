#outcome: ok, but not great. issue was my prompt, I believe.
          Test cases were failing


Prompt: we're going to tackle
dev/issues/0068-todo-for-pe-end-data-we-don-t-need-to-verify-every-read-has-
the-right-name.md. I want you to introduce an option on the [options]
(config/mod.rs:Options struct) that's called 'ValidateReadPairing', defaulting
to true, that if enabled, will add a Validation Step that checks like every
1000ths read for 'proper pairing', using the same logic that ValidateName uses
(using '/' as the read_name_end_char).Do not add the validation step if there's
a ValidateName in the workflow. In the error message, include information how
to turn it off (setting the option to false) and how to change the
readname_end_char (by adding a ValidateName step). Include documentation and
test cases. Set the todo to done at the end.


# ValidateReadPairing Option Implementation Plan

1. Extend `Options` with `spot_check_read_pairing` (default true) and wire it into config parsing + docs.
2. Implement a new validation step `ValidateReadPairing` that samples every 1000th read and reuses `read_name_canonical_prefix` with `/` separator; register it in the transformation enum.
3. Auto-inject the spot-check step when the option is enabled and no explicit `ValidateName` step is present; ensure errors reference the option and ValidateName override.
4. Add regression coverage (unit and fixture-based tests) and update relevant documentation and issue status.