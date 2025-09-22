SELECT sum(unique1) over (rows between unbounded(1) preceding and unbounded(1) following),
       unique1, four
FROM tenk1 WHERE unique1 < 10;
