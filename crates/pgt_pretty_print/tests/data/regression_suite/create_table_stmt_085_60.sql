CREATE TABLE partitioned (
	a int
) PARTITION BY RANGE (immut_func(a));
