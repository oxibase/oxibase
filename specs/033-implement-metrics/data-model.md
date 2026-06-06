# Data Model: System Metrics

## Entity: MetricEvent

Represents an internal database metric to be persisted in `system.metrics`.

- `name` (String): The name of the metric (e.g., `queries_total`).
- `description` (Option<String>): Optional description.
- `unit` (Option<String>): Optional unit (e.g., `ms`, `count`).
- `metric_type` (String): `counter`, `gauge`, or `histogram`.
- `value` (f64): The metric value.
- `attributes` (String): JSON encoded string of key-value attributes.
- `timestamp` (DateTime<Utc>): When the metric was recorded.