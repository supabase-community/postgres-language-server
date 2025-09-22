CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (const_func());
