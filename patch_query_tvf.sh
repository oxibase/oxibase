#!/bin/bash
# First, insert `execute_tvf_source` and helpers inside `impl Executor`
sed -i '' '/fn execute_expression_select/i \
    fn execute_tvf_source(\
        \&self,\
        tvf_source: &FunctionTableSource,\
        stmt: &SelectStatement,\
        ctx: &ExecutionContext,\
        classification: &Arc<QueryClassification>,\
    ) -> SelectResult {\
        let limit_hint = if stmt.where_clause.is_none()\
            && stmt.group_by.columns.is_empty()\
            && stmt.having.is_none()\
            && stmt.order_by.is_empty()\
            && !stmt.distinct\
            && !classification.has_aggregation\
            && !classification.has_window_functions\
            && !classification.has_set_operations\
        {\
            Self::extract_limit_hint(stmt, ctx)\
        } else {\
            None\
        };\
\
        if let Some(ref where_clause) = stmt.where_clause {\
            if let Ok(eval) = ExpressionEval::compile(where_clause, &[]) {\
                if let Ok(val) = eval.with_context(ctx).eval_slice(&[]) {\
                    if val == Value::Boolean(false) {\
                        let columns = if !tvf_source.column_aliases.is_empty() {\
                            tvf_source\
                                .column_aliases\
                                .iter()\
                                .map(|id| id.value.to_string())\
                                .collect()\
                        } else {\
                            vec!["value".to_string()]\
                        };\
                        return self.execute_query_on_memory_result(\
                            stmt,\
                            ctx,\
                            columns,\
                            Vec::new(),\
                        );\
                    }\
                }\
            }\
        }\
\
        let range_hint = Self::extract_tvf_range_hint(tvf_source, stmt, ctx);\
\
        let (result_rows, columns) =\
            Self::evaluate_tvf_with_range(tvf_source, ctx, limit_hint, range_hint)?;\
\
        self.execute_query_on_memory_result(stmt, ctx, columns, result_rows)\
    }\
\
    fn extract_limit_hint(stmt: &SelectStatement, ctx: &ExecutionContext) -> Option<usize> {\
        let limit_expr = stmt.limit.as_ref()?;\
        let offset = stmt.offset.as_ref().and_then(|off| {\
            ExpressionEval::compile(off, &[])\
                .ok()?\
                .with_context(ctx)\
                .eval_slice(&[])\
                .ok()\
                .and_then(|v| v.as_int64())\
                .map(|v| v.max(0) as usize)\
        });\
        let limit_val = ExpressionEval::compile(limit_expr, &[])\
            .ok()?\
            .with_context(ctx)\
            .eval_slice(&[])\
            .ok()?\
            .as_int64()?;\
\
        if limit_val < 0 {\
            return None;\
        }\
        let total = (limit_val as usize).saturating_add(offset.unwrap_or(0));\
        Some(total)\
    }\
\
    fn extract_tvf_range_hint(\
        tvf_source: &FunctionTableSource,\
        stmt: &SelectStatement,\
        ctx: &ExecutionContext,\
    ) -> Option<(Option<i64>, Option<i64>)> {\
        let where_clause = stmt.where_clause.as_ref()?;\
\
        let col_name: String = if !tvf_source.column_aliases.is_empty() {\
            tvf_source.column_aliases[0].value_lower.clone()\
        } else {\
            "value".to_string()\
        };\
\
        let mut min_bound: Option<i64> = None;\
        let mut max_bound: Option<i64> = None;\
\
        Self::collect_range_bounds(where_clause, &col_name, ctx, &mut min_bound, &mut max_bound);\
\
        if min_bound.is_some() || max_bound.is_some() {\
            Some((min_bound, max_bound))\
        } else {\
            None\
        }\
    }\
\
    fn evaluate_tvf_with_range(\
        tvf_source: &FunctionTableSource,\
        ctx: &ExecutionContext,\
        limit: Option<usize>,\
        range_hint: Option<(Option<i64>, Option<i64>)>,\
    ) -> Result<(Vec<Row>, Vec<String>)> {\
        use crate::functions::global_registry;\
\
        let func_name = &tvf_source.function.value;\
        let tvf = global_registry().get_tvf(func_name).ok_or_else(|| {\
            Error::NotSupported(format!("Unknown table-valued function: {}", func_name))\
        })?;\
\
        let mut arg_values = Vec::with_capacity(tvf_source.arguments.len());\
        for arg in &tvf_source.arguments {\
            let value = ExpressionEval::compile(arg, &[])?\
                .with_context(ctx)\
                .eval_slice(&[])?;\
            arg_values.push(value);\
        }\
\
        if let Some((min_bound, max_bound)) = range_hint {\
            if arg_values.len() >= 2 {\
                let all_integer = arg_values.iter().all(|v| matches!(v, Value::Integer(_)));\
                if all_integer {\
                    let start = arg_values[0].as_int64().unwrap();\
                    let stop = arg_values[1].as_int64().unwrap();\
                    let step = if arg_values.len() == 3 {\
                        arg_values[2].as_int64().unwrap()\
                    } else if start <= stop {\
                        1\
                    } else {\
                        -1\
                    };\
\
                    if step == 1 {\
                        if let Some(lo) = min_bound {\
                            if lo > start {\
                                arg_values[0] = Value::Integer(lo);\
                            }\
                        }\
                        if let Some(hi) = max_bound {\
                            if hi < stop {\
                                arg_values[1] = Value::Integer(hi);\
                            }\
                        }\
                    } else if step == -1 {\
                        if let Some(hi) = max_bound {\
                            if hi < start {\
                                arg_values[0] = Value::Integer(hi);\
                            }\
                        }\
                        if let Some(lo) = min_bound {\
                            if lo > stop {\
                                arg_values[1] = Value::Integer(lo);\
                            }\
                        }\
                    }\
                }\
            }\
        }\
\
        let result_rows = tvf.generate(&arg_values, limit)?;\
\
        let columns: Vec<String> = if !tvf_source.column_aliases.is_empty() {\
            tvf_source\
                .column_aliases\
                .iter()\
                .map(|id| id.value.to_string())\
                .collect()\
        } else {\
            tvf.column_names()\
        };\
\
        Ok((result_rows, columns))\
    }\
' src/executor/query.rs

# Second, update execute_table_expression to match Expression::FunctionTableSource
sed -i '' '/Expression::ValuesSource(_)/i \
            Expression::FunctionTableSource(tvf_source) => {\
                let (rows, cols) = Self::evaluate_tvf_with_range(tvf_source, ctx, None, None)?;\
                let result: Box<dyn QueryResult> = Box::new(ExecutorMemoryResult::new(cols.clone(), rows));\
                Ok((result, cols))\
            }\
' src/executor/query.rs

# Then hook it up into execute_select
sed -i '' '/Expression::ValuesSource(values_source) => {/i \
            Expression::FunctionTableSource(tvf_source) => {\
                self.execute_tvf_source(tvf_source, stmt, ctx, classification)\
            }\
' src/executor/query.rs
