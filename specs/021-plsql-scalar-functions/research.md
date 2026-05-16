# Research: PL/SQL Scalar Functions

No unresolved questions or `NEEDS CLARIFICATION` markers were identified in the feature specification.

## Decisions

- **Decision**: Extend existing PL/SQL parser and interpreter to support `RETURN <expr>;`.
- **Rationale**: The feature relies entirely on the already built PL/SQL engine. This natively adds scalar function support with minimal changes and completely avoids external language dependencies like JavaScript or Python for simple inline functions.
- **Alternatives considered**: None, as the requirement is specifically to utilize the native PL/SQL dialect.
