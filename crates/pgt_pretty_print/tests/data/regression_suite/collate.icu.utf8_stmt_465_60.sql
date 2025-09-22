CREATE TABLE test23 (a int, b text COLLATE case_insensitive) PARTITION BY HASH (b);
