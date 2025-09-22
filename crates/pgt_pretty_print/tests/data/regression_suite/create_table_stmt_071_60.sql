CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (retset(a));
