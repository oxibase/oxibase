use strict;
use warnings;

open my $fh, '<', 'src/parser/statements.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

# Extract the sequence methods
my ($methods) = $content =~ m/(    \/\/ ========================================================================\n    \/\/ Sequences\n    \/\/ ========================================================================\n\n    fn parse_create_sequence_statement.*)    }\n}\n/s;

# Remove from end
$content =~ s/    \/\/ ========================================================================\n    \/\/ Sequences\n    \/\/ ========================================================================\n\n    fn parse_create_sequence_statement.*    }\n}\n/}\n/s;

# Insert before mod tests
$content =~ s/\n#\[cfg\(test\)\]/\n$methods\n#\[cfg(test)\]/;

open $fh, '>', 'src/parser/statements.rs' or die $!;
print $fh $content;
close $fh;
