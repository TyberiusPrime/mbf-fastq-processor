status: open
# StoreTagInSequence Optimization

- **Problem**: Currently discards all tag locations when growing/shrinking sequences
- **Solution**: Preserve relevant tag locations during sequence modifications
- **Benefit**: Better tag location tracking throughout pipeline
