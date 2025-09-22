SELECT sum(unique1) over (rows between unbounded.x preceding and unbounded.x following),
       unique1, four
FROM tenk1, (values (1)) as unbounded(x) WHERE unique1 < 10;
