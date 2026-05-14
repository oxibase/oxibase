import re

with open("src/executor/mod.rs", "r") as f:
    content = f.read()

METHODS = """
    /// Initialize system.cron tables if they don't exist
    pub(crate) fn ensure_cron_tables_exist(&self) -> crate::core::Result<()> {
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
    pub(crate) fn execute_internal_sql(&self, sql: &str) -> crate::core::Result<()> {
        let mut parser = crate::parser::Parser::new(sql);
        let program = parser
            .parse_program()
            .map_err(|e| crate::core::Error::parse(e.to_string()))?;

        for stmt in &program.statements {
            let ctx = crate::executor::context::ExecutionContextBuilder::new()
                .with_internal(true)
                .build();
            self.execute_statement(stmt, &ctx)?;
        }

        Ok(())
    }

    /// Initialize system schema, migrate old `_sys_*` tables, and ensure `system.cron` tables exist
    fn ensure_system_schema_and_migrations(&self) -> crate::core::Result<()> {
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
"""

content = content.replace("    fn get_query_planner(&self) -> &QueryPlanner {\n        self.query_planner\n            .get_or_init(|| QueryPlanner::new(Arc::clone(&self.engine)))\n    }", 
                 METHODS + "\n    fn get_query_planner(&self) -> &QueryPlanner {\n        self.query_planner\n            .get_or_init(|| QueryPlanner::new(Arc::clone(&self.engine)))\n    }")

with open("src/executor/mod.rs", "w") as f:
    f.write(content)
