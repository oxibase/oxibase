use strict;
use warnings;

open my $fh, '<', 'src/executor/ddl.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/pub\(crate\) fn execute_create_sequence\(\n        &mut self,/pub(crate) fn execute_create_sequence(\n        &self,/g;
$content =~ s/pub\(crate\) fn execute_alter_sequence\(\n        &mut self,/pub(crate) fn execute_alter_sequence(\n        &self,/g;
$content =~ s/pub\(crate\) fn execute_drop_sequence\(\n        &mut self,/pub(crate) fn execute_drop_sequence(\n        &self,/g;

$content =~ s/crate::executor::result::QueryResult/crate::storage::traits::QueryResult/g;

open $fh, '>', 'src/executor/ddl.rs' or die $!;
print $fh $content;
close $fh;
