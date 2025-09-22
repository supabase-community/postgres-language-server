SELECT COUNT(*) OVER (ORDER BY t1.unique1)
FROM tenk1 t1 INNER JOIN tenk1 t2 ON t1.unique1 = t2.tenthous
LIMIT 1;
