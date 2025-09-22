SELECT DISTINCT amopmethod, amopstrategy, oprname
FROM pg_amop a1 LEFT JOIN pg_operator o1 ON amopopr = o1.oid
ORDER BY 1, 2, 3;
