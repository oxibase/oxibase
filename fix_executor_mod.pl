use strict;
use warnings;

open my $fh, '<', 'src/executor/mod.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/Statement::CreateSequence\(_\) => Err\(Error::internal\("CREATE SEQUENCE not implemented yet"\)\)/Statement::CreateSequence(stmt) => self.execute_create_sequence(stmt)/;
$content =~ s/Statement::AlterSequence\(_\) => Err\(Error::internal\("ALTER SEQUENCE not implemented yet"\)\)/Statement::AlterSequence(stmt) => self.execute_alter_sequence(stmt)/;
$content =~ s/Statement::DropSequence\(_\) => Err\(Error::internal\("DROP SEQUENCE not implemented yet"\)\)/Statement::DropSequence(stmt) => self.execute_drop_sequence(stmt)/;

open $fh, '>', 'src/executor/mod.rs' or die $!;
print $fh $content;
close $fh;
