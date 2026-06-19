# Quickstart: Rhai Timestamp

With this feature, you can use time-based operations inside your Oxibase Rhai procedures.

### Measuring Execution Time

```sql
CREATE PROCEDURE monitor_performance() LANGUAGE rhai AS $$
    let start_time = timestamp();
    
    // ... simulate some heavy work ...
    let sum = 0;
    for i in 0..10000 { sum += i; }
    
    let duration = start_time.elapsed; // returns float in seconds
    return duration;
$$;
```

### Delaying Execution

```sql
CREATE PROCEDURE wait_for_a_bit() LANGUAGE rhai AS $$
    sleep(1.5); // sleep for 1.5 seconds
    return true;
$$;
```