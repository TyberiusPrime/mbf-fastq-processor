## cutadapt
    https://cutadapt.readthedocs.io/en/stable/index.html
    repo: https://github.com/marcelm/cutadapt/
    nixpkgs: no
    flakeable: in imtmarburg/flakes.

    Modifies and filters reads.

### adapter removal

    - can remove 5' or 3' adapters (with partial overlap.).
    - can remove 5' *and* 3' adapters (linked adapters), but only trim when both are present.
    - 'complex' adapter definitions (mas mismatc,h indels, alignment algorithm, iupac)
    - can remove adapters up to n times.
    - can trim so the adapter is cut of, or is cut of after the adapter
    - can mask the adapter sequence with N
    - can mark the adapter by turing it into lowercase
    - rev comp support.

Support: We have 3' adapter trimming in TrimAdapterMismatchTail

Todo: TrimAdapter 5'. 
todo: Trimming retaining adapter.
Todo: Trim requiring both adapters
Todo: FilterAdapterMissing
Todo: Masking
todo: lowercase matches?
todo: Rev comp search support?


### --cut length

Remove fixed number of bases.

Supported. CutStart|CutEnd

## Trim low quality bases

Supported. TrimQualityStart | TrimQualityEnd

## --nextseq-trim Trim dark cycles, trim poly
    Trim polyG with high quality at the end.

Todo: Check if implementation is compatibly with our polyTail trimming.

## --poly-a trim poly-a
Supported.

## trim-n 
Trim polyN from end and start.

Todo: Trim from start.

## --length-tag TAG

Insert a 'TAG=(len)' read 'comment' in the name.

Todo: Should be in our RenameRead step.

## --strip-suffix, -- prefix, --suffix, --rename

Supported: Rename.

template based renaming has the following fields:


    {header} – the full, unchanged header

    {id} – the read ID, that is, the part of the header before the first whitespace

    {comment} – the part of the header after the whitespace following the ID

    {adapter_name} – the name of adapter that was found in this read or no_adapter if there was no adapter match. If you use --times to do multiple rounds of adapter matching, this is the name of the last found adapter.

    {match_sequence} – the sequence of the read that matched the adapter (including errors). If there was no adapter match, this is set to an empty string. If you use a linked adapter, this is to the two matching strings, separated by a comma.

    {cut_prefix} – the prefix removed by the --cut (or -u) option (that is, when used with a positive length argument)

    {cut_suffix} – the suffix removed by the --cut (or -u) option (that is, when used with a negative length argument)

    {rc} – this is replaced with the string rc if the read was reverse complemented. This only applies when reverse complementing was requested.

    \t – not a placeholder, but will be replaced with the tab character.


Todo:  implement templates

##  --zero-cap

Clip negative quality values to zero.

Todo.

## --minimum-length --maximum-length, --max-n

Supported.

## -- max-expected-errors 
Discard reads whose expected number of errors exceeds the value E.
https://cutadapt.readthedocs.io/en/stable/algorithms.html#expected-errors

Todo.

## --discard-trimmed
Discard reads with adapter match.

Todo

## --discard-casava
discard reads that have ':Y:' in their name.
todo: FilterName(regexp...)


## json based report.
Todo: inspect what cutadapt outputs.
Example here:
https://cutadapt.readthedocs.io/en/stable/reference.html#json-report-format

## Can export filtered reads to other files.

Support: no.

## properly paired reads

cutadapt verifies that the names in r1/r2 match.
Todo: add as step


