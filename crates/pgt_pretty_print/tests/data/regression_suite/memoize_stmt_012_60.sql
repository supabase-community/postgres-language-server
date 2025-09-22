SELECT COUNT(*), AVG(t1.twenty) FROM tenk1 t1 LEFT JOIN
LATERAL (SELECT t1.twenty AS c1, t2.unique1 AS c2, t2.two FROM tenk1 t2) s
ON t1.two = s.two
WHERE s.c1 = s.c2 AND t1.unique1 < 1000;
