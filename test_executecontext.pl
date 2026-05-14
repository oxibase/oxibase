use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

if ($content =~ /pub struct ExecuteContext/) {
    print "Found ExecuteContext\n";
}
