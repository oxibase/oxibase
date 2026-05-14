use strict;
use warnings;

open my $fh, '<', 'src/executor/information_schema.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

my $methods = <<'METHODS';
    /// Build information_schema.sequences result
    fn build_sequences_result(&self) -> Result<Box<dyn QueryResult>> {
        let columns = vec![
            "sequence_catalog".to_string(),
            "sequence_schema".to_string(),
            "sequence_name".to_string(),
            "data_type".to_string(),
            "numeric_precision".to_string(),
            "numeric_scale".to_string(),
            "start_value".to_string(),
            "minimum_value".to_string(),
            "maximum_value".to_string(),
            "increment".to_string(),
            "cycle_option".to_string(),
            "current_value".to_string(),
        ];

        let mut rows = Vec::new();
        
        let sequences = self.engine.list_sequences()?;
        
        for (name, options, current_val) in sequences {
            rows.push(Row::from_values(vec![
                Value::text("def"), // catalog
                Value::text("public"), // schema
                Value::text(name),
                Value::text("bigint"), // Assuming all sequences are bigint
                Value::integer(64), // precision
                Value::integer(0), // scale
                Value::integer(options.start_with),
                Value::integer(options.min_value),
                Value::integer(options.max_value),
                Value::integer(options.increment_by),
                Value::text(if options.cycle { "YES" } else { "NO" }),
                Value::integer(current_val),
            ]));
        }

        Ok(Box::new(ExecutorMemoryResult::new(columns, rows)))
    }
METHODS

$content =~ s/    \/\/\/ Build information_schema.sequences result \(empty - no sequences supported\)\n    fn build_sequences_result\(&self\) -> Result<Box<dyn QueryResult>> \{.*?    \}/$methods/s;

open $fh, '>', 'src/executor/information_schema.rs' or die $!;
print $fh $content;
close $fh;
