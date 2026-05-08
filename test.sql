CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT);
BEGIN;
INSERT INTO test_table (id, name) VALUES (1, 'Test');
SELECT state FROM system.transactions;
