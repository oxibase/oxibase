#!/bin/bash
sed -i '' '/ValuesSource(ValuesTableSource),/a \
    \/// Function table source (table-valued function in FROM clause)\
    FunctionTableSource(FunctionTableSource),
' src/parser/ast.rs

sed -i '' '/Expression::ValuesSource(e) => write!(f, "{}", e),/a \
            Expression::FunctionTableSource(e) => write!(f, "{}", e),
' src/parser/ast.rs

sed -i '' '/Expression::ValuesSource(e) => e.token.position,/a \
            Expression::FunctionTableSource(e) => e.token.position,
' src/parser/ast.rs
