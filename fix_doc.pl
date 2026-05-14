use strict;
use warnings;

open my $fh, '<', 'src/executor/ddl.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    \/\/\/ Execute a CREATE SCHEMA statement\n\n    pub\(crate\) fn execute_create_sequence/    \/\/\/ Execute a CREATE SEQUENCE statement\n    pub(crate) fn execute_create_sequence/;

open $fh, '>', 'src/executor/ddl.rs' or die $!;
print $fh $content;
close $fh;
