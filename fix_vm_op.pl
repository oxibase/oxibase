use strict;
use warnings;

open my $fh, '<', 'src/executor/expression/vm.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

my $seq_ops = <<'SEQ_OPS';
                Op::NextVal => {
                    let seq_name = self.stack.pop().unwrap().as_string().unwrap_or_default();
                    let value = if let Some(engine) = ctx.engine {
                        match engine.nextval(&seq_name) {
                            Ok(val) => {
                                crate::executor::context::cache_currval(seq_name, val);
                                Value::Integer(val)
                            }
                            Err(_) => Value::null_unknown(),
                        }
                    } else {
                        Value::null_unknown()
                    };
                    self.stack.push(value);
                    pc += 1;
                }
                
                Op::CurrVal => {
                    let seq_name = self.stack.pop().unwrap().as_string().unwrap_or_default();
                    let value = if let Some(val) = crate::executor::context::get_cached_currval(&seq_name) {
                        Value::Integer(val)
                    } else {
                        Value::null_unknown()
                    };
                    self.stack.push(value);
                    pc += 1;
                }
                
                Op::SetVal => {
                    let is_called = self.stack.pop().unwrap().as_boolean().unwrap_or(true);
                    let value = self.stack.pop().unwrap().as_int64().unwrap_or_default();
                    let seq_name = self.stack.pop().unwrap().as_string().unwrap_or_default();
                    
                    let result = if let Some(engine) = ctx.engine {
                        match engine.setval(&seq_name, value, is_called) {
                            Ok(val) => {
                                // Also update currval for this session
                                crate::executor::context::cache_currval(seq_name, val);
                                Value::Integer(val)
                            }
                            Err(_) => Value::null_unknown(),
                        }
                    } else {
                        Value::null_unknown()
                    };
                    self.stack.push(result);
                    pc += 1;
                }
SEQ_OPS

$content =~ s/(                Op::LoadTransactionId => \{)/$seq_ops\n$1/;

open $fh, '>', 'src/executor/expression/vm.rs' or die $!;
print $fh $content;
close $fh;
