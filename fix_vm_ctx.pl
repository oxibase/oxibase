use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    pub transaction_id: Option<u64>,/    pub transaction_id: Option<u64>,\n\n    \/\/\/ Storage Engine (for NEXTVAL\/SETVAL)\n    pub engine: Option<&'a dyn crate::storage::traits::Engine>,/;

$content =~ s/(transaction_id: None,)/$1\n            engine: None,/g;

open $fh, '>', 'src/executor/expression/vm.rs' or die $!;
print $fh $content;
close $fh;
open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    pub fn with_subquery_executor\(mut self, executor: &'a dyn SubqueryExecutor\) -> Self \{\n        self\.subquery_executor = Some\(executor\);\n        self\n    \}/    pub fn with_subquery_executor(mut self, executor: &'a dyn SubqueryExecutor) -> Self {\n        self.subquery_executor = Some(executor);\n        self\n    }\n\n    \/\/\/ Add storage engine\n    pub fn with_engine(mut self, engine: &'a dyn crate::storage::traits::Engine) -> Self {\n        self.engine = Some(engine);\n        self\n    }/;

open $fh, '>', 'src/executor/expression/vm.rs' or die $!;
print $fh $content;
close $fh;
