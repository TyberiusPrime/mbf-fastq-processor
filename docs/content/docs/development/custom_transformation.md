---
weight: 2
---
# Implementing your own transformation

Let's implement an example step (also called a 'transformation') that converts your reads to FuNkYcAsE
by upper/lower-casing every other letter.

This guide assumes you have basic linux command line knowledge, and that you
can edit text files (source code).


We are going to start by devising a test case, making sure it fails, 
and then step by step adding all the parts we need. This will illustrate
all the infrastructure the project has to support you in this.


## First things first
[Clone the repo]({{< relref "docs/development/getting_started.md" >}})
 and verify that you can build (perhaps
after entering the `nix develop` environment) using `cargo build`.

## Simple test case

To add a test case, we need to add a folder with an input.toml
somewhere below the `test_cases` directory. The input files
used by the test case must be named 'input*', while the output prefix can be anything,
though most existing test cases simply use 'output'. Our test runner will then verify
that mbf-fastq-processor is producing exactly that output.

Create a folder `mkdir test_cases/single_step/funky_case/basic -p`.
We are going to use an existing short FASTQ file as the input,
so change to the just created folder and symlink it using `ln -s ../../../sample_data/misc/input_read1_2.fq input.fq`

Copy it to the expected output `cp input.fq funky_read1.fq` and edit the copy in your editor
to look like this:

```
@test_read1
AgTcAgTcAgTcAgTc
+
IIIIIIIIIIIIIIII
@test_read2
TgAcTgAcTgAcTgAc
+
HHHHHHHHHHHHHHHH
```

Now we need to fill this into input.toml

```toml
[input]
    read1 = 'input.fq'

[[step]]
    action = "FunkyCase"

[output]
    prefix = "funky"

```

Now go back to the top level project directory and run `./dev/update_generated.sh`
which will discover your new test case and add it to our test harness.
(If you omit this step, the all_test_cases_are_generated will fail and remind you
to run that script).

Now run `cargo test` and you should receive the following (expected) failure:


```

---- test_cases_x_single_step_x_funky_case_x_basic stdout ----
Test case is in: test_cases/single_step/funky_case/basic

thread 'test_cases_x_single_step_x_funky_case_x_basic' panicked at mbf-fastq-processor/tests/test_runner.rs:35:9:
Test failed ../test_cases/single_step/funky_case/basic Verification failed:
stderr: Verification failed:

# == Error Details ==
Could not parse toml file: ../test_cases/single_step/funky_case/basic/input.toml

Caused by:
    0: Error in Step 0 (0-based), action = FunkyCase
    1: Something went wrong during deserialization:
       - step[0].action: Unknown variant `FunkyCase`. Did you mean one of `Rename`, `Truncate`, `CutStart`?
	To list available steps, run the `list-steps` command
       in `action`
       

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


## Adding the transformation in all the right places

We need to do three things: 

- Write a [struct](https://doc.rust-lang.org/book/ch05-00-structs.html) & [trait](https://doc.rust-lang.org/book/ch10-02-traits.html) implementation of our transformation in a new rust file
- Hook it into [rust's module system](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- Add it to the central [enum](https://doc.rust-lang.org/book/ch06-00-enums.html) that lists all transformations.

We're going to start with the last step.


### Add a transformation to the central enum that lists all transformations

Edit the file mbf-fastq-processor/src/transformations.rs using your favorite editor.

You are looking for 
```rust
pub enum Transformation {
    //Edits
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    Truncate(edits::Truncate),
    ...
```

And you want to add `FunkyCase(edits::FunkyCase)` so it looks like
```rust
pub enum Transformation {
    //Edits
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    Truncate(edits::Truncate),
    FunkyCase(edits::FunkyCase),
    ...
```

`edits::` in this case refers to a module below the `transformation` module, 
which brings os to our next step:


### Hook a step into the module system.


We are going to tell the `edits` module that it has a submodule `funky_case`,
and reexport one type called `FunkyCase' from `funky_case` so that the 
rest of the rust code can use it.

Open `mbf-fastq-processor/src/transformations/edits.rs` and add

```rust
mod funky_case; // declare that we have a module funky_case(.rs)

pub use funky_case::FunkyCase; //export our struct
```

### Write the transformation

A transformation is a struct that implements the 'Step' trait.

To be usable, your struct needs to be included in the large `Transformation` enum listing all steps,
which we accomplished in the previous steps.

Now it's time to actually write the struct. Create a new file
`mbf-fastq-processor/src/transformations/edits/funky_case.rs`
and put the following minimal example into it;

```rust
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunkyCase {
}

impl Step for FunkyCase {
    fn apply(
        &self,
        mut block: FastQBlocksCombined, // that's where the read data lives
        _input_info: &InputInfo,        //ignore for now
        _block_no: usize,               //ignore for now
        _demultiplex_info: &OptDemultiplex, //ignore for now
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        //this doesn't do anything.

        Ok((block, true))
    }
}
```

At this point `cargo check` should show no error (but a warning about `mut block` not
needing the mut because it's not being changed. Ignore that for now, we're going to
to alter reads soon).

Our test case however will now fail with a different message:

```
---- test_cases_x_single_step_x_funky_case_x_basic stdout ----
Test case is in: test_cases/single_step/funky_case/basic

thread 'test_cases_x_single_step_x_funky_case_x_basic' panicked at mbf-fastq-processor/tests/test_runner.rs:35:9:
Test failed ../test_cases/single_step/funky_case/basic Verification failed:
stderr: Verification failed:

# == Error Details ==
Output verification failed:
  funky_read1.fq: Content mismatch at byte 13: expected 0x67, got 0x47

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Which is to be expected, since we're not actually changing the reads yet.

You can compare the expected and actual output. For failing test cases, after a run,
there is a folder 'actual' in the test case directory that has the produced output.

You can simply compare them with diff

```
>cd test_cases/single_step/funky_case/basic
> diff actual/funky_read1.fq funky_read1.fq 
2c2
< AGTCAGTCAGTCAGTC
---
> AgTcAgTcAgTcAgTc
6c6
< TGACTGACTGACTGAC
---
> TgAcTgAcTgAcTgAc
```

### Modifying the reads

To actually change the reads, we are going to use a function
that takes a callback that modifies each read in turn.

Replace the contents of `mbf-fastq-processor/src/transformations/edits/funky_case.rs` with this
```rust
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunkyCase {}

impl Step for FunkyCase {
    fn apply(
        &self,
        mut block: FastQBlocksCombined, // that's where the read data lives
        _input_info: &InputInfo,        //ignore for now
        _block_no: usize,               //ignore for now
        _demultiplex_info: &OptDemultiplex, //ignore for now
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        //apply funky casing to all reads
        block.apply_in_place_wrapped(
            SegmentIndex(0),  // in segment one, see below for configurabitlity
            |read| { // a lambda function taking a WrappedFastQRead mutable reference
            let mut lower = true; //so we can alternate
            for char in read.seq_mut().iter_mut() { //for every character in the sequence
                if lower {
                    *char = char.to_ascii_lowercase()
                } else {
                    *char = char.to_ascii_uppercase()
                }
                lower = !lower;
            }
        }, 
        None // if_tag support, see below
        );

        Ok((block, true))
    }
}
```

If you run `cargo test` now, our funky case test will pass,
and it will fail later on with the tests that verify our documentation:

```
---- test_every_transformation_has_documentation stdout ----

thread 'test_every_transformation_has_documentation' panicked at mbf-fastq-processor/tests/template_and_documentation_verification.rs:937:9:
The following transformations are missing documentation files:
FunkyCase
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test_every_transformation_has_benchmark stdout ----

thread 'test_every_transformation_has_benchmark' panicked at mbf-fastq-processor/tests/template_and_documentation_verification.rs:1322:9:
The following transformations are missing benchmarks in simple_benchmarks.rs:
FunkyCase

---- test_llm_guide_covers_all_transformations stdout ----

thread 'test_llm_guide_covers_all_transformations' panicked at mbf-fastq-processor/tests/template_and_documentation_verification.rs:1136:9:
LLM guide validation failed:
Transformation 'FunkyCase' is not documented in llm-guide.md

---- test_every_step_has_a_template_section stdout ----

thread 'test_every_step_has_a_template_section' panicked at mbf-fastq-processor/tests/template_and_documentation_verification.rs:820:5:
Template validation failed:
The following transformations are missing in template.toml:
FunkyCase
```

### Making the transformation configurable.

We'll add this later, for now let's make FunkyCase configurable.

We want it to work on any segment (not just the first one), support `if_tag` like 
all the other read editing transformations and allow starting with either a lower or upper 
case letter.

Let's start with adding a mandatory boolean flag that decides whether 
we start with a lowercase letter or not.

Edit our test case and add the flag:

```toml
[input]
    read1 = 'input.fq'

[[step]]
    action = "FunkyCase"
    start_with_lower = true # new flag

[output]
    prefix = "funky"
```

Then copy the test case for the reverse case:
`cp  test_cases/single_step/funky_case/basic test_cases/single_step/funky_case/upper_first -r`
and edit `start_with_lower = true` to `start_with_lower = false` in that test case's input.tom.

Don't forget to change the expected output by replacing
`test_cases/single_step/funky_case/upper_first/funky_read.fq`
```FASTQ
@test_read1
aGtCaGtCaGtCaGtC
+
IIIIIIIIIIIIIIII
@test_read2
tGaCtGaCtGaCtGaC
+
HHHHHHHHHHHHHHHH
```

Let it find the new test (`./dev/update_generated.sh`) and watch both of them fail with
`cargo test`: 

```
# shown for only one of them.
--- test_cases_x_single_step_x_funky_case_x_basic stdout ----
Test case is in: test_cases/single_step/funky_case/basic

thread 'test_cases_x_single_step_x_funky_case_x_basic' panicked at mbf-fastq-processor/tests/test_runner.rs:35:9:
Test failed ../test_cases/single_step/funky_case/basic Verification failed:
stderr: Verification failed:

# == Error Details ==
Could not parse toml file: ../test_cases/single_step/funky_case/basic/input.toml

Caused by:
    0: Error in Step 0 (0-based), action = FunkyCase
    1: Something went wrong during deserialization:
       - step[0]: unknown field `start_with_lower`, there are no fields
```

Go back to `funky_case.rs` and replace it with

```rust
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunkyCase {
    start_with_lower: bool,
}

impl Step for FunkyCase {
    fn apply(
        &self,
        mut block: FastQBlocksCombined, // that's where the read data lives
        _input_info: &InputInfo,        //ignore for now
        _block_no: usize,               //ignore for now
        _demultiplex_info: &OptDemultiplex, //ignore for now
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        //apply funky casing to all reads
        block.apply_in_place_wrapped(
            SegmentIndex(0), // in segment one, see below for configurabitlity
            |read| {
                // a lambda function taking a WrappedFastQRead mutable reference
                let mut lower = self.start_with_lower; //so we can alternate
                for char in read.seq_mut().iter_mut() {
                    //for every character in the sequence
                    if lower {
                        *char = char.to_ascii_lowercase()
                    } else {
                        *char = char.to_ascii_uppercase()
                    }
                    lower = !lower;
                }
            },
            None, // if_tag support, see below
        );

        Ok((block, true))
    }
}
```

The step test cases will now pass, while the documentation test cases will still fail.


### If_tag and segment support


For a full transformation we're still missing two ingredients:
the choice of what segment to work on, and the `if_tag`  support allowing
it to work on a subset of reads.

We're going to do both at once now.

Duplicate the basic test case once more: 
```
cp  test_cases/single_step/funky_case/basic test_cases/single_step/funky_case/if_tag_segment -r
```

Place the following in   `test_cases/single_step/funky_case/if_tag_segment/funky_read2.fq`
```fastq
@test_read1
AgTcAgTcAgTcAgTc
+
IIIIIIIIIIIIIIII
@test_read2
TGACTGACTGACTGAC
+
HHHHHHHHHHHHHHHH
```


Place the following in input.toml
```
options.accept_duplicate_files = true
[input]
    read1 = 'input.fq'
    read2 = 'input.fq'


[[step]]
    action = "FunkyCase"
    start_with_lower = true
    segment = 'read1'

[[step]]
  action ="EvalExpression"
  out_label = "apply_funky_to_read2"
  expression = "read_no < 1"
  result_type = "bool"

[[step]]
    action = "FunkyCase"
    start_with_lower = false
    segment = 'read2'
    if_tag = 'apply_funky_to_read2'

[output]
    prefix = "funky"
```

Update the tests once more with `./dev/update_generated.sh` and observe it failing, because we
haven't added the options yet:

```
---- test_cases_x_single_step_x_funky_case_x_if_tag_segment stdout ----
Test case is in: test_cases/single_step/funky_case/if_tag_segment

thread 'test_cases_x_single_step_x_funky_case_x_if_tag_segment' panicked at mbf-fastq-processor/tests/test_runner.rs:35:9:
Test failed ../test_cases/single_step/funky_case/if_tag_segment Verification failed:
stderr: Verification failed:

# == Error Details ==
Could not parse toml file: ../test_cases/single_step/funky_case/if_tag_segment/input.toml

Caused by:
    0: Error in Step 0 (0-based), action = FunkyCase
    1: Something went wrong during deserialization:
       - step[0]: unknown field `segment`, expected `start_with_lower`
       - step[1].action: Unknown variant `EvalExpr`. Did you mean one of `EvalExpression`, `Swap`, `CalcKmers`?
	To list available steps, run the `list-steps` command
       in `action`
       - step[2]: unknown field `if_tag`, expected `start_with_lower`
```


Open `mbf-fastq-processor/src/transformations/edits/funky_case.rs` and modify it to

```rust
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunkyCase {
    start_with_lower: bool,

    #[serde(default)] // accept omission iff exactly one Segment is definied in config
    segment: SegmentOrAll,

    #[serde(default)]
    #[serde(skip)] // do not read this from configuration
    segment_index: Option<SegmentIndexOrAll>, // the internal representation after validation

    #[serde(default)]
    if_tag: Option<String>, // defaults to 'None' if omitted
}

impl Step for FunkyCase {
    fn uses_tags(
        //inform the framework about the tags the step uses
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>, //only relevant for Steps that have no
                                                         //user-defined set of tags to process
    ) -> Option<Vec<(String, &[TagValueType])>> {
        // runs during config validation
        self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            vec![(
                cond_tag.tag.clone(),
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            )]
        })
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        // runs during config validation
        // convert the segment name to our internal index representation
        // also makes sure we have a valid segment
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined, // that's where the read data lives
        _input_info: &InputInfo,        //ignore for now
        _block_no: usize,               //ignore for now
        _demultiplex_info: &OptDemultiplex, //ignore for now
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // we need to generate a bool vector for the if_tag condition
        let condition = self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            get_bool_vec_from_tag(&block, &cond_tag)
        });

        //apply funky casing to all reads with if_tag
        block.apply_in_place_wrapped_plus_all(
            // also accept 'all', and then apply to all segments
            // by calling the function multiple times
            self.segment_index
                .expect("Segment index set in validate_segments"), 
            |read| {
                // a lambda function taking a `WrappedFastQRead` mutable reference
                let mut lower = self.start_with_lower; //so we can alternate
                for char in read.seq_mut().iter_mut() {
                    //for every character in the sequence
                    if lower {
                        *char = char.to_ascii_lowercase()
                    } else {
                        *char = char.to_ascii_uppercase()
                    }
                    lower = !lower;
                }
            },
            condition.as_deref(), // if_tag support
        );

        Ok((block, true))
    }
}
```

At this stage you have a working 'Step' (verify with `cargo test`) that has all
the usual amenities, but lacks documentation.

To add this is left as an exercise for the reader,
but you'll need to edit `mbf-fastq-processor/src/template.toml`,
`docs/content/docs/reference/llm-guide.md` and 
add a file `docs/content/docs/reference/modification-steps/FunkyCase.md` 
which need to include a valid TOML block with `action = "FunkyCase"` and all available options documented.

You'll also need to add a microbenchmark to 
`mbf-fastq-processor/benches/simple_benchmarks.rs`.

Congratulations, you just wrote your first transformation for mbf-fastq-processor!

