SELECT t1.oid, t1.typname, p1.oid, p1.proname
FROM pg_type AS t1, pg_proc AS p1
WHERE t1.typinput = p1.oid AND t1.typtype in ('b', 'p') AND NOT
    (t1.typelem != 0 AND t1.typlen < 0) AND NOT
    (p1.prorettype = t1.oid AND NOT p1.proretset)
ORDER BY 1;
