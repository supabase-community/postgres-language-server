CREATE TABLE test22 (a int, b text COLLATE case_sensitive) PARTITION BY HASH (b);
