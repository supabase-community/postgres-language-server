CREATE TABLE test21 (a int, b text COLLATE case_insensitive) PARTITION BY RANGE (b);
