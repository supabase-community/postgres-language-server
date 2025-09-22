CREATE TABLE test22a (a int, b text[] COLLATE case_sensitive) PARTITION BY HASH (b);
