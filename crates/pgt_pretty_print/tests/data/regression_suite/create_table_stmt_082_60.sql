CREATE TABLE partitioned (
	a int,
	b int
) PARTITION BY RANGE (((a, b)));
