use strict;
use warnings;

open my $fh, '<', 'src/storage/mvcc/engine.rs' or die $!;
my @lines = <$fh>;
close $fh;

my $start = 1820;
my $brace_count = 0;
my $found = 0;
my $end = -1;

for my $i ($start - 1 .. $#lines) {
    my $line = $lines[$i];
    $brace_count += () = $line =~ /\{/g;
    $brace_count -= () = $line =~ /\}/g;
    if ($brace_count > 0) {
        $found = 1;
    }
    if ($found && $brace_count == 0) {
        $end = $i;
        print "End of impl Engine is at line: ", $i + 1, "\n";
        last;
    }
}
