status: open
# Read Overlap Detection (BD Rhapsody Style)

- **Algorithm**: Modified Knuth-Morris-Pratt substring search
- **Parameters**:
  - Maximum mismatch rate: 9% (configurable)
  - Minimum overlap length: 25 bases
- **Process**:
  1. Scan read1 right-to-left on reverse complement of read2
  2. Find closest offset with lowest mismatches below threshold
  3. Split merged read according to R1 minimum length + bead capture sequence length
- **Benefit**: Prevent downstream mis-alignment and mis-assembly
- **Metrics**: Calculate overlap detection percentage for troubleshooting 
