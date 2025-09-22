SELECT o1.oid, o1.oprname, p1.oid, p1.proname
FROM pg_operator AS o1, pg_proc AS p1
WHERE o1.oprcode = p1.oid AND
    (o1.oprcanmerge OR o1.oprcanhash) AND
    p1.provolatile = 'v';
