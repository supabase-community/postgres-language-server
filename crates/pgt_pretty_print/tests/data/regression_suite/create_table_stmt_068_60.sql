CREATE TABLE partitioned (
	a int
) INHERITS (some_table) PARTITION BY LIST (a);
