use strict;
use warnings;

open my $fh, '<', 'src/parser/statements.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

# Replace ALTER map
$content =~ s/"ALTER" => self\.parse_alter_statement\(\)\.map\(Statement::AlterTable\)/"ALTER" => self.parse_alter_statement()/;

# Add CREATE SEQUENCE to parse_create_statement
$content =~ s/(} else if self\.peek_token_is_keyword\("SCHEMA"\) \{)/} else if self.peek_token_is_keyword("SEQUENCE") {\n            self.next_token();\n            self.parse_create_sequence_statement().map(Statement::CreateSequence)\n        $1/;

# Add DROP SEQUENCE to parse_drop_statement
$content =~ s/(} else if self\.peek_token_is_keyword\("SCHEMA"\) \{)/} else if self.peek_token_is_keyword("SEQUENCE") {\n            self.next_token();\n            self.parse_drop_sequence_statement().map(Statement::DropSequence)\n        $1/g;

# Refactor parse_alter_statement
my $new_alter = <<'NEW_ALTER';
    fn parse_alter_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if self.peek_token_is_keyword("SEQUENCE") {
            self.next_token();
            return self.parse_alter_sequence_statement().map(Statement::AlterSequence);
        }

        // Expect TABLE
        if !self.expect_keyword("TABLE") {
            return None;
        }

        self.parse_alter_table_statement(token).map(Statement::AlterTable)
    }

    fn parse_alter_table_statement(&mut self, token: Token) -> Option<AlterTableStatement> {
NEW_ALTER

$content =~ s/    fn parse_alter_statement\(&mut self\) -> Option<AlterTableStatement> \{\n        let token = self\.cur_token\.clone\(\);\n\n        \/\/ Expect TABLE\n        if !self\.expect_keyword\("TABLE"\) \{\n            return None;\n        \}/$new_alter/;

open $fh, '>', 'src/parser/statements.rs' or die $!;
print $fh $content;
close $fh;
