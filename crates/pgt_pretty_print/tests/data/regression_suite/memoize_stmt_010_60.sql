SELECT COUNT(*), AVG(t1.twenty) FROM tenk1 t1 LEFT JOIN
LATERAL (SELECT t1.two+1 AS c1, t2.unique1 AS c2 FROM tenk1 t2) s ON TRUE
WHERE s.c1 = s.c2 AND t1.unique1 < 1000;
