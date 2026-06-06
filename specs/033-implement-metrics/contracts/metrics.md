# Contracts: System Metrics

No external API contracts. Internally, developers emit metrics using the `tracing` macros:

```rust
tracing::info!(
    target: "oxibase::metrics",
    metric_type = "counter",
    metric_name = "queries_total",
    value = 1.0,
    unit = "count",
    description = "Total number of executed queries"
);
```