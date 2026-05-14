import re

with open("src/executor/dml.rs", "r") as f:
    content = f.read()

content = content.replace("        if Schema::is_reserved_namespace(&table_name_raw) {\n            return Err(Error::ReservedNamespaceModification(table_name_raw));\n        }",
                 "        if Schema::is_reserved_namespace(&table_name_raw) && !ctx.is_internal() {\n            return Err(Error::ReservedNamespaceModification(table_name_raw));\n        }")

with open("src/executor/dml.rs", "w") as f:
    f.write(content)
