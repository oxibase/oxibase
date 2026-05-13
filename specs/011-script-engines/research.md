# Phase 0: Research Findings

## Topic 1: Dynamic Property Interception in RustPython
**Decision**: Implement `__getattr__` and `__setattr__` using `#[pymethod(magic)]` inside a `#[pyclass]` definition.
**Rationale**: `rustpython_vm` exposes Python's magic methods directly to Rust structs via the `#[pymethod(magic)]` macro. By implementing `getattr` (which receives the property name as a `PyStrRef`), we can read the dynamic column name, look it up in the `CURRENT_SCHEMA` thread-local, and return the corresponding Python object (e.g., `vm.ctx.new_int`). Similarly, `setattr` allows us to intercept mutations (`NEW.balance = 5`), coerce the incoming Python object to our native Rust `DataType`, and mutate the underlying pointer safely. If a column doesn't exist, we return `Err(vm.new_attribute_error(...))` to behave exactly like a native Python object.
**Alternatives considered**: Using `PyDict` and copying all values eagerly. This was rejected because it violates the "Zero-Copy" and memory efficiency mandates established in feature `010-event-triggers`.

## Topic 2: Dynamic Property Interception in Boa (JavaScript)
**Decision**: Use `boa_engine::object::builtins::JsProxy::builder` to create native `get` and `set` proxy traps.
**Rationale**: Boa's API natively supports creating ECMAScript `Proxy` objects directly from Rust. By providing native Rust closures for `.get(row_get)` and `.set(row_set)`, we can intercept JS property access (e.g., `NEW.column_name`). Inside these closures, we resolve the property key from `args[1]` and interface directly with the thread-local state. Returning `JsValue::new(true)` from the setter trap cleanly signals a successful property assignment in JS strict mode.
**Alternatives considered**: Injecting a raw `new Proxy(target, handler)` string to be evaluated by the VM. This was rejected as it's slower, error-prone, and clutters the trigger execution context with helper code. The native builder is cleaner and significantly more performant.
