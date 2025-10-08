#outcome: success. Even though this plan is terribly short, it did one shot this.
# Extract IUPAC With Indel

1. Extend `dna` utilities with an indel-aware IUPAC finder and expose it through the IO layer.
2. Add a new `ExtractIUPACWithIndel` transformation wiring plus configuration surface updates.
3. Format sources and run focused tests to confirm the new alignment logic.
