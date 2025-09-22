CREATE TABLE test23a (a int, b text[] COLLATE case_insensitive) PARTITION BY HASH (b);
