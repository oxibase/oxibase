use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    pub engine: Option<&'a dyn crate::storage::traits::Engine>,\n//g;
$content =~ s/    \/\/\/ Storage Engine \(for NEXTVAL\/SETVAL\)\n//g;
$content =~ s/            engine: None,\n//g;

open $fh, '>', 'src/executor/expression/vm.rs' or die $!;
print $fh $content;
close $fh;
