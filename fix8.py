import re

with open("src/executor/mod.rs", "r") as f:
    content = f.read()

content = content.replace("Statement::Call(stmt) => self.execute_call(stmt, &ctx),", "Statement::Call(stmt) => self.execute_call(stmt, &ctx),\n            Statement::CreateSchedule(stmt) => self.execute_create_schedule(stmt, &ctx),\n            Statement::AlterSchedule(stmt) => self.execute_alter_schedule(stmt, &ctx),\n            Statement::DropSchedule(stmt) => self.execute_drop_schedule(stmt, &ctx),")

with open("src/executor/mod.rs", "w") as f:
    f.write(content)
