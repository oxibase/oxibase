use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/let value = if let Some\(engine\) = ctx\.engine \{/let value = if let Some(engine) = crate::executor::context::get_current_engine() {/g;
$content =~ s/let result = if let Some\(engine\) = ctx\.engine \{/let result = if let Some(engine) = crate::executor::context::get_current_engine() {/g;

open $fh, '>', 'src/executor/expression/vm.rs' or die $!;
print $fh $content;
close $fh;
