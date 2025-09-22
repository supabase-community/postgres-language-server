SELECT o1.oid, o1.oprname, o2.oid, o2.oprname
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oprcom = o2.oid AND
    (o1.oprcanmerge != o2.oprcanmerge OR
     o1.oprcanhash != o2.oprcanhash);
