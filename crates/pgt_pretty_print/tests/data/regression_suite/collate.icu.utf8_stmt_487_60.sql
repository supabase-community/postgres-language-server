CREATE TABLE test32 (a int, b char(3) COLLATE case_sensitive) PARTITION BY HASH (b);
