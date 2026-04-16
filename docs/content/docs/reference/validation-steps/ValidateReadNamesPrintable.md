# ValidateReadNamesPrintable

Validate that every read name conforms to the SAM/BAM specification.

The SAM specification requires that query names (QNAME) match `[!-?A-~]{1,254}`:
printable ASCII characters excluding `@` and space, with a maximum length of 254.

```toml
[[step]]
    action = "ValidateReadNamesPrintable"
```

No additional parameters are needed — the allowed character set is fixed by the SAM spec.

## When to use

Add this step when your pipeline writes non-BAM output that will become BAM eventually
and you suspect read names may contain invalid characters.

If you're outputing BAM, this is being checked anyway.


## Error example

A FASTQ file with `@@InvalidAtSign` as a name line produces:

```
ValidateReadNamesPrintable: Read name '@InvalidAtSign' contains a character not allowed
by the SAM/BAM spec (byte 0x40).
Allowed characters: [!-?A-~] (printable ASCII, excluding '@' and space).
```

## See also

- The BAM writer performs the same check at output time; this step lets you detect the
  problem earlier in the pipeline.
- Use [`Rename`]({{< relref "docs/reference/modification-steps/Rename.md" >}}) to fix
  read names that contain invalid characters.
