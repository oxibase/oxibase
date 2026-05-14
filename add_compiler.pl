use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/compiler.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

my $seq_ops = <<'SEQ_OPS';
            "NEXTVAL" if func.arguments.len() == 1 => {
                self.compile_expr(&func.arguments[0], builder)?;
                builder.emit(Op::NextVal);
                return Ok(());
            }

            "CURRVAL" if func.arguments.len() == 1 => {
                self.compile_expr(&func.arguments[0], builder)?;
                builder.emit(Op::CurrVal);
                return Ok(());
            }

            "SETVAL" if func.arguments.len() == 2 || func.arguments.len() == 3 => {
                self.compile_expr(&func.arguments[0], builder)?;
                self.compile_expr(&func.arguments[1], builder)?;
                if func.arguments.len() == 3 {
                    self.compile_expr(&func.arguments[2], builder)?;
                } else {
                    builder.emit(Op::LoadValue(crate::core::Value::Boolean(true)));
                }
                builder.emit(Op::SetVal);
                return Ok(());
            }

SEQ_OPS

$content =~ s/("CURRENT_TRANSACTION_ID" => \{)/$seq_ops$1/;

open $fh, '>', 'src/executor/expression/compiler.rs' or die $!;
print $fh $content;
close $fh;
