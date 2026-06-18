# Research: Rhai Timestamp

## Decision: Utilize Default Rhai Packages
**Rationale**: Rhai's `Engine::new()` automatically loads the standard library, which includes both the `BasicTimePackage` (providing `timestamp()`, `elapsed`, `+`, `-`) and `LanguageCorePackage` (providing `sleep()`). We simply need to ensure that the engine hasn't been instantiated via `Engine::new_raw()` or had standard packages removed, and that we don't compile `rhai` with the `no_time` feature. Upon investigating `src/functions/backends/rhai.rs`, the engine is instantiated using `Engine::new()`. Therefore, these functions are actually natively available out of the box in the `rhai` scripting backend of Oxibase. The only task required is verifying this through comprehensive tests.
**Alternatives considered**: 
- Manually registering custom `timestamp()` and `sleep()` native Rust functions (Rejected as redundant since standard Rhai packages already cover this).
