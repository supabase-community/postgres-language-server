CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (xmin);
