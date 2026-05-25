# Research

## Diagram Generation Setup
- **Decision**: Use the modified `railroad-diagrams` implementation from DuckDB (or directly from Tab Atkins Jr. depending on specific needs) and structure syntax definitions in a modular way similar to DuckDB's `js/1.3/` directory structure.
- **Rationale**: The specification explicitly mandates using the DuckDB javascript library for client-side SVG generation. It's a proven approach for SQL documentation.
- **Alternatives considered**: 
  - Static generation at build time (rejected by spec).
  - Manual image creation (rejected by spec).

## Integration with Documentation Site
- **Decision**: Since Oxibase is a Rust project, we need to locate where the documentation lives (likely a `docs/` folder using `mdbook` or a similar tool). We need to ensure the chosen tool allows embedding raw HTML/JS or can be extended to support custom JS execution for the diagrams.
- **Rationale**: The generated diagrams must be integrated into the final output of the documentation site.
- **Alternatives considered**: None, as this depends on the existing repo setup.