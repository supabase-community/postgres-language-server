SELECT COUNT(*) OVER ()
FROM tenk1 t1 INNER JOIN tenk1 t2 ON t1.unique1 = t2.tenthous
WHERE t2.two = 1
LIMIT 1;
