use strict;
use warnings;

open my $fh, '<', 'src/parser/statements.rs' or die $!;
my $content = do { local $/; <$fh> };
close $fh;

my $tests = <<'TESTS';
    #[test]
    fn test_parse_create_sequence() {
        let input = "CREATE SEQUENCE seq1 START WITH 100 INCREMENT BY -10 MINVALUE -1000 MAXVALUE 1000 CYCLE";
        let stmt = parse_stmt(input).unwrap();
        if let Statement::CreateSequence(s) = stmt {
            assert_eq!(s.name.to_string(), "seq1");
            assert_eq!(s.start_with, Some(100));
            assert_eq!(s.increment_by, Some(-10));
            assert_eq!(s.min_value, Some(-1000));
            assert_eq!(s.max_value, Some(1000));
            assert!(s.cycle);
        } else {
            panic!("Expected CreateSequence");
        }
    }

    #[test]
    fn test_parse_alter_sequence() {
        let input = "ALTER SEQUENCE seq1 RESTART WITH 50 INCREMENT BY 5 NO MINVALUE CYCLE";
        let stmt = parse_stmt(input).unwrap();
        if let Statement::AlterSequence(s) = stmt {
            assert_eq!(s.name.to_string(), "seq1");
            assert_eq!(s.restart_with, Some(50));
            assert_eq!(s.increment_by, Some(5));
            assert_eq!(s.cycle, Some(true));
        } else {
            panic!("Expected AlterSequence");
        }
    }

    #[test]
    fn test_parse_drop_sequence() {
        let input = "DROP SEQUENCE IF EXISTS seq1";
        let stmt = parse_stmt(input).unwrap();
        if let Statement::DropSequence(s) = stmt {
            assert_eq!(s.name.to_string(), "seq1");
            assert!(s.if_exists);
        } else {
            panic!("Expected DropSequence");
        }
    }
TESTS

$content =~ s/}\n$/$tests}\n/;

open $fh, '>', 'src/parser/statements.rs' or die $!;
print $fh $content;
close $fh;
