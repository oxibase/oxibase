# Data Model: Scripting Backend Context Refactor

This feature does not introduce any new database tables, internal core data structures, logical execution nodes, or physical AST changes. 

The scope is strictly limited to the translation layer sitting between Oxibase's `Row` struct and the dynamic scripting environments representations (`NewRowProxy`, `PyDict`, `JsObject`).