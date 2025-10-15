status: done
# Filter Inversion Consistency

- **Problem**: Inconsistent inversion support across filters
  - Some filters can invert (e.g., `FilterOtherFile`)
  - Others are inverses of each other (e.g., `FilterMinLen`, `FilterMaxLen`)
- **Solution**: Add consistent `invert` flag to all filters
- **Benefit**: Cleaner, more intuitive filter configuration
