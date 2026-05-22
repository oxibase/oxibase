#!/bin/bash
sed -i '' '/use super::window::{/i \
use super::tvf::{GenerateSeriesFunction, GenerateSeriesScalarFunction, TableValuedFunction};\
' src/functions/registry.rs

sed -i '' '/registry.register_window::<RowNumberFunction>();/i \
        // Register generate_series as scalar (returns JSON array for SELECT usage)\
        registry.register_scalar::<GenerateSeriesScalarFunction>();\
' src/functions/registry.rs

sed -i '' '/registry.register_window::<CumeDistFunction>();/a \
\
        // Register built-in table-valued functions\
        registry.register_tvf(\
            "GENERATE_SERIES",\
            Arc::new(|| Box::new(GenerateSeriesFunction)),\
        );\
' src/functions/registry.rs
