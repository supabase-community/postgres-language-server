SELECT p1.oid::regprocedure, p2.oid::regprocedure
FROM pg_proc AS p1, pg_proc AS p2
WHERE p1.oid < p2.oid AND p1.proname = p2.proname AND
    p1.prokind = 'a' AND p2.prokind = 'a' AND
    array_dims(p1.proargtypes) != array_dims(p2.proargtypes)
ORDER BY 1;
