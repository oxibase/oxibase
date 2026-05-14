            Statement::CreateSchedule(stmt) => self.execute_create_schedule(stmt, &ctx),
            Statement::AlterSchedule(stmt) => self.execute_alter_schedule(stmt, &ctx),
            Statement::DropSchedule(stmt) => self.execute_drop_schedule(stmt, &ctx),
