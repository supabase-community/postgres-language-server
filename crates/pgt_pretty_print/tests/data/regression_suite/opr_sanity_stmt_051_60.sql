SELECT o1.oid, o1.oprname
FROM pg_operator as o1
WHERE (o1.oprkind != 'b' AND o1.oprkind != 'l') OR
    o1.oprresult = 0 OR o1.oprcode = 0;
