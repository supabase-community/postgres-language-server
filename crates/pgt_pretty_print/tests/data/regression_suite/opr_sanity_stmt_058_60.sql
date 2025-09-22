SELECT o1.oid, o1.oprname FROM pg_operator AS o1
WHERE (o1.oprcanmerge OR o1.oprcanhash) AND NOT
    (o1.oprkind = 'b' AND o1.oprresult = 'bool'::regtype AND o1.oprcom != 0);
