use strict;
use warnings;

open my $fh, '<', 'src/core/error.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    SchemaNotFound\(String\),/    SchemaNotFound(String),\n\n    \/\/\/ Sequence already exists\n    #[error("sequence already exists: {0}")]\n    SequenceAlreadyExists(String),\n\n    \/\/\/ Sequence not found\n    #[error("sequence not found: {0}")]\n    SequenceNotFound(String),/s;

open $fh, '>', 'src/core/error.rs' or die $!;
print $fh $content;
close $fh;
