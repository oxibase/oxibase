# Quickstart: OpenTelemetry Tracing

## Enabling External Tracing (OTLP)

To export query lifecycle traces to an external observability platform (like Jaeger, Zipkin, or Datadog), set the standard OpenTelemetry environment variables before starting the database:

```bash
# Set the gRPC endpoint for your OTLP collector
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"

# Optionally set the service name
export OTEL_SERVICE_NAME="oxibase"

# Start the database
cargo run --release --features cli
```

Once started, the database will automatically forward tracing spans (parsing, planning, execution) to the configured endpoint.

## Querying Traces Internally

Even without an external exporter configured, the database captures its own traces and writes them to the `system.traces` table.

You can query your own query performance using standard SQL:

```sql
SELECT 
    name, 
    duration_ms, 
    attributes 
FROM system.traces 
ORDER BY start_time DESC 
LIMIT 10;
```

This is useful for identifying slow queries or understanding where time is spent (e.g., is the parser slow, or is the execution slow?).
