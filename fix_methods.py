import re

with open("src/executor/mod.rs", "r") as f:
    content = f.read()

METHODS = """
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
"""

content = content.replace("    fn get_query_planner(&self) -> &QueryPlanner {\n        self.query_planner\n            .get_or_init(|| QueryPlanner::new(Arc::clone(&self.engine)))\n    }", 
                 METHODS + "\n    fn get_query_planner(&self) -> &QueryPlanner {\n        self.query_planner\n            .get_or_init(|| QueryPlanner::new(Arc::clone(&self.engine)))\n    }")

with open("src/executor/mod.rs", "w") as f:
    f.write(content)
