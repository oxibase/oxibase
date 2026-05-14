use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    \/\/\/ Add storage engine\n    pub fn with_engine\(mut self, engine: &'a dyn crate::storage::traits::Engine\) -> Self \{\n        self.engine = Some\(engine\);\n        self\n    \}\n//g;

open $fh, '>', 'src/executor/expression/vm.rs' or die $!;
print $fh $content;
close $fh;
