CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE ((avg(a)));
