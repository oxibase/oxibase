use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/program.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/Op::CaseEnd => 0,/Op::CaseEnd => 0,\n                Op::NextVal | Op::CurrVal => 0,\n                Op::SetVal => -2,/g;

open $fh, '>', 'src/executor/expression/program.rs' or die $!;
print $fh $content;
close $fh;
