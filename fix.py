import re

with open("src/executor/mod.rs", "r") as f:
    content = f.read()

# 1. Imports
content = content.replace("pub use context::{ExecutionContext, TimeoutGuard};", 
                          "pub use context::{ExecutionContext, ExecutionContextBuilder, TimeoutGuard};")

# 2. Add ensure_system_schema_and_migrations inside Executor::new
content = content.replace("let _ = executor.load_functions();", 
                          "let _ = executor.ensure_system_schema_and_migrations();\n        let _ = executor.load_functions();")

# 3. Add to execute_statement
content = content.replace("Statement::Call(stmt) => self.execute_call(stmt, &ctx),",
                          "Statement::Call(stmt) => self.execute_call(stmt, &ctx),\n            Statement::CreateSchedule(stmt) => self.execute_create_schedule(stmt, &ctx),\n            Statement::AlterSchedule(stmt) => self.execute_alter_schedule(stmt, &ctx),\n            Statement::DropSchedule(stmt) => self.execute_drop_schedule(stmt, &ctx),")

# 4. Add methods at the end of impl Executor
METHODS = """
    /// Initialize system schema, migrate old `_sys_*` tables, and ensure `system.cron` tables exist
    fn ensure_system_schema_and_migrations(&self) -> Result<()> {
        // Ensure system schema exists
        self.execute_internal_sql("CREATE SCHEMA IF NOT EXISTS system;")?;

        // Run migrations for old _sys_ tables to system.*
        self.execute_internal_sql("CREATE TABLE IF NOT EXISTS system.procedures AS SELECT * FROM _sys_procedures;")
            .ok();
        self.execute_internal_sql("DROP TABLE IF EXISTS _sys_procedures;")
            .ok();

        self.execute_internal_sql("CREATE TABLE IF NOT EXISTS system.functions AS SELECT * FROM _sys_functions;")
            .ok();
        self.execute_internal_sql("DROP TABLE IF EXISTS _sys_functions;")
            .ok();

        self.execute_internal_sql("CREATE TABLE IF NOT EXISTS system.triggers AS SELECT * FROM _sys_triggers;")
            .ok();
        self.execute_internal_sql("DROP TABLE IF EXISTS _sys_triggers;")
            .ok();

        self.execute_internal_sql("CREATE TABLE IF NOT EXISTS system.table_stats AS SELECT * FROM _sys_table_stats;")
            .ok();
        self.execute_internal_sql("DROP TABLE IF EXISTS _sys_table_stats;")
            .ok();

        self.execute_internal_sql("CREATE TABLE IF NOT EXISTS system.column_stats AS SELECT * FROM _sys_column_stats;")
            .ok();
        self.execute_internal_sql("DROP TABLE IF EXISTS _sys_column_stats;")
            .ok();

        // Ensure cron tables exist
        self.ensure_cron_tables_exist()?;

        Ok(())
    }

    /// Initialize system.cron tables if they don't exist
    pub(crate) fn ensure_cron_tables_exist(&self) -> Result<()> {
        use crate::storage::jobs::{
            CREATE_CRON_RUNS_SQL, CREATE_CRON_SQL, SYS_CRON, SYS_CRON_RUNS,
        };

        let tx = self.engine.begin_transaction()?;
        let tables = tx.list_tables()?;
        let has_cron = tables.iter().any(|t| t.eq_ignore_ascii_case(SYS_CRON));
        let has_cron_runs = tables
            .iter()
            .any(|t| t.eq_ignore_ascii_case(SYS_CRON_RUNS));
        drop(tx);

        if !has_cron {
            self.execute_internal_sql(CREATE_CRON_SQL)?;
        }

        if !has_cron_runs {
            self.execute_internal_sql(CREATE_CRON_RUNS_SQL)?;
        }

        Ok(())
    }

    /// Execute internal SQL (e.g., system table creation)
    pub(crate) fn execute_internal_sql(&self, sql: &str) -> Result<()> {
        let mut parser = crate::parser::Parser::new(sql);
        let program = parser
            .parse_program()
            .map_err(|e| Error::parse(e.to_string()))?;

        for stmt in &program.statements {
            let ctx = ExecutionContextBuilder::new()
                .with_internal(true)
                .build();
            self.execute_statement(stmt, &ctx)?;
        }

        Ok(())
    }
}
"""

content = re.sub(r'    fn begin\(&self\) -> crate::core::Result<\(\)> \{\n        // No-op for procedures\n        Ok\(\)\n    }\n\}',
                 "    fn begin(&self) -> crate::core::Result<()> {\n        // No-op for procedures\n        Ok(())\n    }\n" + METHODS,
                 content)

with open("src/executor/mod.rs", "w") as f:
    f.write(content)
