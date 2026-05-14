use strict;
use warnings;

open my $fh, '<', 'src/parser/statements.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/!self\.cur_token_is\(TokenType::Eof\)\n            && !\(self\.cur_token_is\(TokenType::Punctuator\) && self\.cur_token\.literal == ";"\)/!(self.cur_token_is(TokenType::Eof) || self.cur_token_is(TokenType::Punctuator) && self.cur_token.literal == ";")/g;

$content =~ s/                if self\.peek_token_is_keyword\("MINVALUE"\) \{\n                    self\.next_token\(\);\n                \} else if self\.peek_token_is_keyword\("MAXVALUE"\) \{\n                    self\.next_token\(\);\n                \} else if/                if self.peek_token_is_keyword("MINVALUE") || self.peek_token_is_keyword("MAXVALUE") {\n                    self.next_token();\n                } else if/g;

open $fh, '>', 'src/parser/statements.rs' or die $!;
print $fh $content;
close $fh;
