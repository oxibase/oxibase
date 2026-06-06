#!/bin/bash
cargo run -- repl -d memory:// << 'SQL'
CREATE TABLE test (id INT);
INSERT INTO test VALUES (1);
INSERT INTO test VALUES (2);
SELECT COUNT(*) FROM test;
SELECT COUNT(id) FROM test;
SQL
