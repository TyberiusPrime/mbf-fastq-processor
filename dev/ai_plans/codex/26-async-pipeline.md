# Async pipeline redesign

- Replace thread-based pipeline stages with async tasks.
- Share work via Tokio channels to preserve block ordering.
- Keep error collection/report output logic equivalent for future benchmarks.
