---
weight: 1
---

# Philosophy


Mbf-fastq-processor transforms (DNA) sequencing reads for downstream analysis.

Its focus are on
 - correctness
 - reproducibility
 - a lack of surprises
 - friendliness
 - speed


## Correctness

We strive to do the right thing, always.

To that end, Mbf-fastq-processor is tested with more than 500
end-to-end, input-to-output tests, both during development and via
continuous integration.

## Reproducibility

Repeated runs on the same bits (input data & configuration)
must deliver the same output bits. Every time.

*Anything else is a bug*.

We allow ourselves one, perhaps useful, exception:

The reports (JSON and HTML ) by default leak information about the runtime environment,
such as the exact input configuration, the names of the input files,
the working directory and the version of mbf-fastq-processor used.

## A lack of surprises

mbf-fastq-processor does exactly what you define in it's configuration TOML to your reads.

It does not trim, filter, or mangle them in any way without an explicit `[[step]]`.

And every knob that is not absolutely obvious must be set explicitly in the configuration.

This makes getting started harder (but the user's going to be copy/pasting configuration
anyway), but prevents situations where an update changed a default and now your
output is different and you have no idea where to start looking.


## Friendliness

Every error message should contain actionable advice where to look next.

Every error message should contain as much information as necessary.

mbf-fastq-processor tests everything it can about your configuration
before actually starting to read your data, and if an error is detected
during run, it terminates swiftly. And the implementation can detect
more than one configuration error at once.

We also clearly distinguish between unrecoverable-but-foreseen errors
(invalid configurations, broken input files) and bugs - invalid conditions
where mbf-fastq-processor's internal contracts were broken.

The later still produce a 'friendly panic' message with instructions
on how to get help.

## Speed

There's two aspects to such a tool's speed.
One is a general 'less entropy used' = 'better for everyone'.

The other one is a usage issue: if your feedback loop time is high,
you can not iterate ideas and configurations efficiently.

While we're not continuously measuring performance, we have 
[a benchmark framework]({{< relref "docs/development/benchmarking.md" >}})
in place and include [performance benchmark results]({{< relref "docs/reference/benchmark-section.md" >}}#results),
so users have an idea what to expect.







