use strict;
use warnings;

open my $fh, '<', 'src/storage/mvcc/engine.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

my $methods = <<'METHODS';
    // --- Sequences ---

    fn sequence_exists(&self, sequence_name: &str) -> Result<bool> {
        let sequences = self.sequences.read().unwrap();
        Ok(sequences.contains_key(&sequence_name.to_lowercase()))
    }

    fn create_sequence(&self, sequence_name: &str, options: crate::core::SequenceOptions) -> Result<()> {
        let mut sequences = self.sequences.write().unwrap();
        let name_lower = sequence_name.to_lowercase();
        if sequences.contains_key(&name_lower) {
            return Err(crate::core::Error::already_exists(format!("Sequence '{}' already exists", sequence_name)));
        }
        sequences.insert(name_lower, std::sync::Arc::new(crate::core::SequenceState::new(options)));
        Ok(())
    }

    fn alter_sequence(&self, sequence_name: &str, options: crate::core::SequenceOptions) -> Result<()> {
        let mut sequences = self.sequences.write().unwrap();
        let name_lower = sequence_name.to_lowercase();
        if !sequences.contains_key(&name_lower) {
            return Err(crate::core::Error::not_found(format!("Sequence '{}' does not exist", sequence_name)));
        }
        // Create new sequence state and replace existing
        sequences.insert(name_lower, std::sync::Arc::new(crate::core::SequenceState::new(options)));
        Ok(())
    }

    fn drop_sequence(&self, sequence_name: &str) -> Result<()> {
        let mut sequences = self.sequences.write().unwrap();
        let name_lower = sequence_name.to_lowercase();
        if sequences.remove(&name_lower).is_none() {
            return Err(crate::core::Error::not_found(format!("Sequence '{}' does not exist", sequence_name)));
        }
        Ok(())
    }

    fn nextval(&self, sequence_name: &str) -> Result<i64> {
        let sequence = {
            let sequences = self.sequences.read().unwrap();
            let name_lower = sequence_name.to_lowercase();
            if let Some(seq) = sequences.get(&name_lower) {
                std::sync::Arc::clone(seq)
            } else {
                return Err(crate::core::Error::not_found(format!("Sequence '{}' does not exist", sequence_name)));
            }
        };
        sequence.nextval()
    }

    fn setval(&self, sequence_name: &str, value: i64, is_called: bool) -> Result<i64> {
        let sequence = {
            let sequences = self.sequences.read().unwrap();
            let name_lower = sequence_name.to_lowercase();
            if let Some(seq) = sequences.get(&name_lower) {
                std::sync::Arc::clone(seq)
            } else {
                return Err(crate::core::Error::not_found(format!("Sequence '{}' does not exist", sequence_name)));
            }
        };
        sequence.setval(value, is_called)
    }

    fn list_sequences(&self) -> Result<Vec<(String, crate::core::SequenceOptions, i64)>> {
        let sequences = self.sequences.read().unwrap();
        let mut result = Vec::with_capacity(sequences.len());
        for (name, seq) in sequences.iter() {
            result.push((name.clone(), seq.options.clone(), seq.current_value()));
        }
        Ok(result)
    }
METHODS

$content =~ s/(    fn fetch_rows_by_ids.*\n    \})/$1\n$methods/;

open $fh, '>', 'src/storage/mvcc/engine.rs' or die $!;
print $fh $content;
close $fh;
