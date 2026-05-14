use strict;
use warnings;

open my $fh, '<', 'src/storage/mvcc/engine.rs' or die $!;
my @lines = <$fh>;
close $fh;

splice(@lines, 277, 0, "    /// Sequence definitions\n    pub(crate) sequences: Arc<RwLock<FxHashMap<String, Arc<crate::core::SequenceState>>>>,\n");

open $fh, '>', 'src/storage/mvcc/engine.rs' or die $!;
print $fh join("", @lines);
close $fh;
