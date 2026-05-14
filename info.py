import re

with open("src/executor/information_schema.rs", "r") as f:
    content = f.read()

content = content.replace("            // Skip internal system schema tables\n            if schema_name.eq_ignore_ascii_case(\"system\") || schema_name.eq_ignore_ascii_case(\"information_schema\") {\n                continue;\n            }", "            // Skip internal system schema tables and old _sys_ prefixed tables\n            if schema_name.eq_ignore_ascii_case(\"system\") \n                || schema_name.eq_ignore_ascii_case(\"information_schema\")\n                || actual_table_name.starts_with(\"_sys_\") \n            {\n                continue;\n            }")
content = content.replace("let sql = \"SELECT schema, name, parameters, return_type FROM _sys_functions ORDER BY name\";", "let sql = \"SELECT schema, name, parameters, return_type FROM system.functions ORDER BY name\";")

with open("src/executor/information_schema.rs", "w") as f:
    f.write(content)
