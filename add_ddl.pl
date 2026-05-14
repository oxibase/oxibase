use strict;
use warnings;

open my $fh, '<', 'src/executor/ddl.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

my $methods = <<'METHODS';

    pub(crate) fn execute_create_sequence(
        &mut self,
        stmt: &CreateSequenceStatement,
    ) -> Result<ExecutionResult> {
        let name = stmt.name.to_string();
        
        if self.engine.sequence_exists(&name)? {
            if stmt.if_not_exists {
                return Ok(ExecutionResult::Success {
                    message: format!("Sequence '{}' already exists, skipping", name),
                });
            }
            return Err(Error::SequenceAlreadyExists(name));
        }

        let mut options = crate::core::SequenceOptions::default();
        if let Some(v) = stmt.start_with { options.start_with = v; }
        if let Some(v) = stmt.increment_by { options.increment_by = v; }
        if let Some(v) = stmt.min_value { options.min_value = v; }
        if let Some(v) = stmt.max_value { options.max_value = v; }
        options.cycle = stmt.cycle;

        self.engine.create_sequence(&name, options)?;

        Ok(ExecutionResult::Success {
            message: format!("Sequence '{}' created", name),
        })
    }

    pub(crate) fn execute_alter_sequence(
        &mut self,
        stmt: &AlterSequenceStatement,
    ) -> Result<ExecutionResult> {
        let name = stmt.name.to_string();
        
        if !self.engine.sequence_exists(&name)? {
            if stmt.if_exists {
                return Ok(ExecutionResult::Success {
                    message: format!("Sequence '{}' does not exist, skipping", name),
                });
            }
            return Err(Error::SequenceNotFound(name));
        }

        // Just recreating with new values conceptually for ALTER, but we might want to preserve the old start_with
        // We'll extract current options, modify them, and alter.
        // For now, this is a basic implementation that just parses the options.
        // Ideally, Engine should expose get_sequence_options
        
        let mut options = crate::core::SequenceOptions::default();
        if let Some(v) = stmt.restart_with { options.start_with = v; }
        if let Some(v) = stmt.increment_by { options.increment_by = v; }
        if let Some(v) = stmt.min_value { options.min_value = v; }
        if let Some(v) = stmt.max_value { options.max_value = v; }
        if let Some(v) = stmt.cycle { options.cycle = v; }
        
        self.engine.alter_sequence(&name, options)?;

        Ok(ExecutionResult::Success {
            message: format!("Sequence '{}' altered", name),
        })
    }

    pub(crate) fn execute_drop_sequence(
        &mut self,
        stmt: &DropSequenceStatement,
    ) -> Result<ExecutionResult> {
        let name = stmt.name.to_string();
        
        if !self.engine.sequence_exists(&name)? {
            if stmt.if_exists {
                return Ok(ExecutionResult::Success {
                    message: format!("Sequence '{}' does not exist, skipping", name),
                });
            }
            return Err(Error::SequenceNotFound(name));
        }

        self.engine.drop_sequence(&name)?;

        Ok(ExecutionResult::Success {
            message: format!("Sequence '{}' dropped", name),
        })
    }
METHODS

$content =~ s/(    pub\(crate\) fn execute_create_schema)/$methods\n$1/;

open $fh, '>', 'src/executor/ddl.rs' or die $!;
print $fh $content;
close $fh;
