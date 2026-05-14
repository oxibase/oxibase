use strict;
use warnings;

open my $fh, '<', 'src/executor/ddl.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/-> Result<ExecutionResult> {/-> Result<Box<dyn crate::executor::result::QueryResult>> {/g;
$content =~ s/Ok\(ExecutionResult::Success \{\n[ \t]*message: format!\("[^"]*", name\),\n[ \t]*\}\)/Ok(Box::new(crate::executor::result::ExecResult::new(0, 0)))/g;

open $fh, '>', 'src/executor/ddl.rs' or die $!;
print $fh $content;
close $fh;
