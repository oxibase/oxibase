use strict;
use warnings;

open my $fh, '<', 'src/parser/statements.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

# Wait, let's see where the unclosed delimiter is.
# Before `#[cfg(test)]`, we need to make sure `impl Parser` is closed.
# Actually, the methods I inserted might not be properly closed or I replaced the `impl Parser` closing brace!
# Let's just fix it manually.
