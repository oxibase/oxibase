# 1. Replace the context import
sed -i '' 's/pub use context::{ExecutionContext, TimeoutGuard};/pub use context::{ExecutionContext, ExecutionContextBuilder, TimeoutGuard};/g' src/executor/mod.rs

# 2. Add ensure_system_schema_and_migrations inside Executor::new
sed -i '' 's/let _ = executor.load_functions();/let _ = executor.ensure_system_schema_and_migrations();\
        let _ = executor.load_functions();/' src/executor/mod.rs

# 3. Add the execute_create_schedule methods in execute_statement
awk '
/Statement::DropTrigger/ {
    print "            Statement::DropTrigger(stmt) => self.execute_drop_trigger(stmt, &ctx),"
    print "            Statement::CreateSchedule(stmt) => self.execute_create_schedule(stmt, &ctx),"
    print "            Statement::AlterSchedule(stmt) => self.execute_alter_schedule(stmt, &ctx),"
    print "            Statement::DropSchedule(stmt) => self.execute_drop_schedule(stmt, &ctx),"
    next
}
{ print }
' src/executor/mod.rs > temp.rs && mv temp.rs src/executor/mod.rs

# 4. Add the methods at the end of impl Executor (before fn begin)
awk '
/fn begin\(&self\) -> crate::core::Result<\(\)> {/ {
    print "    /// Initialize system schema, migrate old `_sys_*` tables, and ensure `system.cron` tables exist"
    print "    fn ensure_system_schema_and_migrations(&self) -> Result<()> {"
    print "        self.execute_internal_sql(\"CREATE SCHEMA IF NOT EXISTS system;\")?;"
    print "        self.execute_internal_sql(\"CREATE TABLE IF NOT EXISTS system.procedures AS SELECT * FROM _sys_procedures;\").ok();"
    print "        self.execute_internal_sql(\"DROP TABLE IF EXISTS _sys_procedures;\").ok();"
    print "        self.execute_internal_sql(\"CREATE TABLE IF NOT EXISTS system.functions AS SELECT * FROM _sys_functions;\").ok();"
    print "        self.execute_internal_sql(\"DROP TABLE IF EXISTS _sys_functions;\").ok();"
    print "        self.execute_internal_sql(\"CREATE TABLE IF NOT EXISTS system.triggers AS SELECT * FROM _sys_triggers;\").ok();"
    print "        self.execute_internal_sql(\"DROP TABLE IF EXISTS _sys_triggers;\").ok();"
    print "        self.execute_internal_sql(\"CREATE TABLE IF NOT EXISTS system.table_stats AS SELECT * FROM _sys_table_stats;\").ok();"
    print "        self.execute_internal_sql(\"DROP TABLE IF EXISTS _sys_table_stats;\").ok();"
    print "        self.execute_internal_sql(\"CREATE TABLE IF NOT EXISTS system.column_stats AS SELECT * FROM _sys_column_stats;\").ok();"
    print "        self.execute_internal_sql(\"DROP TABLE IF EXISTS _sys_column_stats;\").ok();"
    print "        self.ensure_cron_tables_exist()?;"
    print "        Ok(())"
    print "    }"
    print ""
    print "    pub(crate) fn ensure_cron_tables_exist(&self) -> Result<()> {"
    print "        use crate::storage::jobs::*;"
    print "        let tx = self.engine.begin_transaction()?;"
    print "        let tables = tx.list_tables()?;"
    print "        let has_cron = tables.iter().any(|t| t.eq_ignore_ascii_case(SYS_CRON));"
    print "        let has_cron_runs = tables.iter().any(|t| t.eq_ignore_ascii_case(SYS_CRON_RUNS));"
    print "        drop(tx);"
    print "        if !has_cron { self.execute_internal_sql(CREATE_CRON_SQL)?; }"
    print "        if !has_cron_runs { self.execute_internal_sql(CREATE_CRON_RUNS_SQL)?; }"
    print "        Ok(())"
    print "    }"
    print ""
    print "    pub(crate) fn execute_internal_sql(&self, sql: &str) -> Result<()> {"
    print "        let mut parser = crate::parser::Parser::new(sql);"
    print "        let program = parser.parse_program().map_err(|e| Error::parse(e.to_string()))?;"
    print "        for stmt in &program.statements {"
    print "            let ctx = ExecutionContextBuilder::new().with_internal(true).build();"
    print "            self.execute_statement(stmt, &ctx)?;"
    print "        }"
    print "        Ok(())"
    print "    }"
    print ""
}
{ print }
' src/executor/mod.rs > temp.rs && mv temp.rs src/executor/mod.rs
